use std::{collections::HashMap, path::Path};

use serde::Deserialize;

use crate::features::dps_scan::{errors::ScanError, models::DependencyType};

// ─── CycloneDX SBOM types ────────────────────────────────────────────────────

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

// ─── Parsing & classification ─────────────────────────────────────────────────

pub fn parse_sbom(path: &Path) -> Result<CycloneDxBom, ScanError> {
    let json = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&json)?)
}

/// Returns (purls, dep_type_map) — all library PURLs and which are direct.
pub fn classify_dependencies(bom: &CycloneDxBom) -> (Vec<String>, HashMap<String, DependencyType>) {
    // Find the root bom-ref from metadata
    let root_ref = bom
        .metadata
        .as_ref()
        .and_then(|m| m.component.as_ref())
        .and_then(|c| c.bom_ref.as_deref());

    let mut dep_types: HashMap<String, DependencyType> = HashMap::new();

    if let Some(root) = root_ref {
        if let Some(root_dep) = bom.dependencies.iter().find(|d| d.r#ref == root) {
            for direct_purl in &root_dep.depends_on {
                dep_types.insert(direct_purl.clone(), DependencyType::Direct);
            }
        }
    }

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
        .filter(|p| {
            !EXCLUDE_PREFIXES.iter().any(|prefix| p.starts_with(prefix))
        }) // skip GitHub Actions
        .collect();

    // Anything not in direct set is transitive
    for purl in &purls {
        dep_types
            .entry(purl.clone())
            .or_insert(DependencyType::Transitive);
    }

    (purls, dep_types)
}
