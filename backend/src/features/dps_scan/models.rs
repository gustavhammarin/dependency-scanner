use serde::{Deserialize, Serialize};

use crate::features::sources::PackageSource;

#[derive(Deserialize)]
pub struct DpsScanRequest {
    pub package_id: String,
    pub package_version: String,
    #[serde(default)]
    pub source: PackageSource,
}

/// Final result returned to the client.
#[derive(Serialize, Deserialize, Debug)]
pub struct DpsScanResult {
    pub total_scanned: usize,
    pub vulnerable_count: usize,
    pub direct_count: usize,
    pub transitive_count: usize,
    pub findings: Vec<DependencyFinding>,
}

/// One dependency that has at least one vulnerability.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DependencyFinding {
    pub purl: String,
    pub name: String,
    pub version: String,
    pub ecosystem: String,
    pub dependency_type: DependencyType,
    /// Full OSV vulnerability objects (same shape as GET /v1/vulns/{id}).
    pub vulnerabilities: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DependencyType {
    Direct,
    Transitive,
}
