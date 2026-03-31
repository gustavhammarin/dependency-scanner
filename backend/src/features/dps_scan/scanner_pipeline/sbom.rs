use std::{collections::HashMap, path::Path};


use serde::Deserialize;

use crate::features::dps_scan::{errors::ScanError, models::DependencyType};

#[derive(Deserialize)]
pub struct CycloneDxBom {
    pub metadata: Option<CdxMetadata>,
    #[serde(default)]
    pub components: Vec<CdxComponent>,
    #[serde(default)]
    pub dependencies: Vec<CdxDependency>,
}

#[derive(Deserialize)]
pub struct CdxMetadata {
    pub component: Option<CdxComponent>,
}

#[derive(Deserialize)]
pub struct CdxComponent {
    #[serde(rename = "bom-ref")]
    pub bom_ref: Option<String>,
    #[serde(rename = "type")]
    pub component_type: Option<String>,
    pub purl: Option<String>,
}

#[derive(Deserialize)]
pub struct CdxDependency {
    #[serde(rename = "ref")]
    pub r#ref: String,
    #[serde(rename = "dependsOn", default)]
    pub depends_on: Vec<String>,
}

const EXCLUDE_PREFIXES: &[&str] = &[
    "pkg:github/",
    "pkg:pypi/pip@",
    "pkg:pypi/wheel@",
    "pkg:pypi/setuptools@",
];


pub fn parse_sbom(path: &Path) -> Result<CycloneDxBom, ScanError> {
    let json = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&json)?)
}

pub fn classify_dependencies(bom: &CycloneDxBom) -> (Vec<String>, HashMap<String, DependencyType>) {
    let root_ref = bom
        .metadata
        .as_ref()
        .and_then(|m| m.component.as_ref())
        .and_then(|c| c.bom_ref.as_deref());

    let direct_refs: Vec<String> = root_ref
        .and_then(|root| bom.dependencies.iter().find(|d| d.r#ref == root))
        .map(|dep| dep.depends_on.clone())
        .unwrap_or_default();

    let purls: Vec<String> = bom
        .components
        .iter()
        .filter(|c| {
            c.component_type
                .as_deref()
                .map(|t| t == "library")
                .unwrap_or(true)
        })
        .filter_map(|c| c.purl.clone())
        .filter(|p| !EXCLUDE_PREFIXES.iter().any(|prefix| p.starts_with(prefix)))
        .collect();

    let dep_types: HashMap<String, DependencyType> = purls
        .iter()
        .map(|purl| {
            let kind = if direct_refs.iter().any(|r| purl_matches(purl, r)) {
                DependencyType::Direct
            } else {
                DependencyType::Transitive
            };
            (purl.clone(), kind)
        })
        .collect();

    (purls, dep_types)
}

fn purl_matches(purl: &str, dep_ref: &str) -> bool {
    if purl.eq_ignore_ascii_case(dep_ref) {
        return true;
    }
    let purl_tail = purl
        .split_once('/')
        .map(|(_, t)| t)
        .unwrap_or(purl)
        .to_lowercase();
    let dep_lower = dep_ref.to_lowercase();

    purl_tail == dep_lower
        || purl_tail.contains(&dep_lower)
        || dep_lower.contains(&purl_tail)
}
