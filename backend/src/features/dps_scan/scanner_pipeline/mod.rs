mod osv;
mod purl;
mod sbom;

use std::{collections::HashSet, path::Path};

use super::{
    errors::ScanError,
    models::{DependencyFinding, DependencyType, DpsScanResult},
};

/// Scan a source directory using a pre-generated `cdx.json` (CycloneDX SBOM).
/// The file is expected to have been created during package download.
pub async fn scan_directory(dir: &Path) -> Result<DpsScanResult, ScanError> {
    let sbom_path = dir.join("cdx.json");
    run_sbom_pipeline(&sbom_path).await
}

/// Shared pipeline: parse a CycloneDX SBOM file, classify deps, query OSV,
/// fetch full vuln details, and assemble DpsScanResult.
pub(super) async fn run_sbom_pipeline(bom_path: &Path) -> Result<DpsScanResult, ScanError> {
    let bom = sbom::parse_sbom(bom_path)?;
    let (purls, dep_types) = sbom::classify_dependencies(&bom);

    if purls.is_empty() {
        tracing::info!("[dps-scan] No packages found in SBOM");
        return Ok(DpsScanResult {
            total_scanned: 0,
            vulnerable_count: 0,
            direct_count: 0,
            transitive_count: 0,
            findings: vec![],
        });
    }

    tracing::info!("[dps-scan] SBOM parsed: {} packages", purls.len());

    let vuln_ids_per_purl = osv::query_osv_batch(&purls).await?;

    let unique_ids: HashSet<String> = vuln_ids_per_purl
        .iter()
        .flatten()
        .flatten()
        .cloned()
        .collect();

    tracing::info!(
        "[dps-scan] OSV found {} unique vulnerabilities across {} packages",
        unique_ids.len(),
        vuln_ids_per_purl
            .iter()
            .filter(|v| !v.as_ref().map(|i| i.is_empty()).unwrap_or(true))
            .count()
    );

    let vuln_details = osv::fetch_vuln_details(unique_ids.into_iter().collect()).await;

    let mut findings: Vec<DependencyFinding> = purls
        .iter()
        .zip(vuln_ids_per_purl.iter())
        .filter_map(|(p, ids_opt)| {
            let ids = ids_opt.as_ref()?;
            if ids.is_empty() {
                return None;
            }
            let vulns: Vec<serde_json::Value> = ids
                .iter()
                .filter_map(|id| vuln_details.get(id).cloned())
                .collect();
            if vulns.is_empty() {
                return None;
            }
            let (name, version, ecosystem) = purl::parse_purl(p);
            let dependency_type = dep_types
                .get(p)
                .cloned()
                .unwrap_or(DependencyType::Transitive);
            Some(DependencyFinding {
                purl: p.clone(),
                name,
                version,
                ecosystem,
                dependency_type,
                vulnerabilities: vulns,
            })
        })
        .collect();

    findings.sort_by(|a, b| {
        let type_ord = matches!(b.dependency_type, DependencyType::Direct)
            .cmp(&matches!(a.dependency_type, DependencyType::Direct));
        type_ord.then_with(|| a.name.cmp(&b.name))
    });

    let total_scanned = purls.len();
    let vulnerable_count = findings.len();
    let direct_count = findings
        .iter()
        .filter(|f| f.dependency_type == DependencyType::Direct)
        .count();
    let transitive_count = vulnerable_count - direct_count;

    Ok(DpsScanResult {
        total_scanned,
        vulnerable_count,
        direct_count,
        transitive_count,
        findings,
    })
}
