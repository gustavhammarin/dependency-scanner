use std::collections::HashMap;

use futures::StreamExt;
use serde::{Deserialize, Serialize};

use crate::features::dps_scan::errors::ScanError;

#[derive(Serialize)]
struct OsvBatchRequest {
    queries: Vec<OsvQuery>,
}

#[derive(Serialize)]
struct OsvQuery {
    package: OsvPurl,
}

#[derive(Serialize)]
struct OsvPurl {
    purl: String,
}

#[derive(Deserialize)]
struct OsvBatchResponse {
    results: Vec<OsvQueryResult>,
}

#[derive(Deserialize)]
struct OsvQueryResult {
    #[serde(default)]
    vulns: Vec<OsvVulnRef>,
}

#[derive(Deserialize)]
struct OsvVulnRef {
    id: String,
}


/// Returns a Vec aligned 1:1 with `purls`. Each entry is Some(ids) or None.
pub async fn query_osv_batch(purls: &[String]) -> Result<Vec<Option<Vec<String>>>, ScanError> {
    let client = reqwest::Client::builder()
        .user_agent("3pp-analyzer/1.0")
        .build()?;

    let mut all_results: Vec<Option<Vec<String>>> = Vec::with_capacity(purls.len());

    // OSV batch API accepts max 999 queries per request
    for chunk in purls.chunks(999) {
        let queries: Vec<OsvQuery> = chunk
            .iter()
            .map(|p| OsvQuery {
                package: OsvPurl { purl: p.clone() },
            })
            .collect();

        let resp = client
            .post("https://api.osv.dev/v1/querybatch")
            .json(&OsvBatchRequest { queries })
            .send()
            .await?
            .error_for_status()?
            .json::<OsvBatchResponse>()
            .await?;

        for result in resp.results {
            let ids: Vec<String> = result.vulns.into_iter().map(|v| v.id).collect();
            all_results.push(if ids.is_empty() { None } else { Some(ids) });
        }
    }

    Ok(all_results)
}

pub async fn fetch_vuln_details(ids: Vec<String>) -> HashMap<String, serde_json::Value> {
    let client = reqwest::Client::builder()
        .user_agent("3pp-analyzer/1.0")
        .build()
        .expect("Failed to build HTTP client");

    futures::stream::iter(ids)
        .map(|id| {
            let c = client.clone();
            async move {
                let url = format!("https://api.osv.dev/v1/vulns/{id}");
                match c.get(&url).send().await {
                    Ok(resp) if resp.status().is_success() => {
                        match resp.json::<serde_json::Value>().await {
                            Ok(json) => Some((id, json)),
                            Err(e) => {
                                tracing::warn!("[dps-scan] Failed to parse vuln {id}: {e}");
                                None
                            }
                        }
                    }
                    Ok(resp) => {
                        tracing::warn!("[dps-scan] OSV returned {} for vuln {id}", resp.status());
                        None
                    }
                    Err(e) => {
                        tracing::warn!("[dps-scan] Failed to fetch vuln {id}: {e}");
                        None
                    }
                }
            }
        })
        .buffer_unordered(10)
        .filter_map(futures::future::ready)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect()
}
