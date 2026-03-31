use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::features::{package_service::models::Package, sources::PackageSource};

#[derive(Serialize, Debug, Deserialize)]
pub struct InsertPackageSchema {
    pub cache_path: String,
    pub package_id: String,
    pub version: String,
    pub package_source: PackageSource,
    pub fetch_date: chrono::DateTime<Utc>,
    pub size_bytes: i64
}

#[derive(Serialize, Debug, Deserialize)]
pub struct GetPackageSchema{
    pub package_id: String,
    pub version: String,
    pub package_source: PackageSource
}

#[derive(Serialize, Debug, Deserialize)]
pub struct PackageResponseSchema{
    pub id: i64,
    pub package_id: String,
    pub version: String,
    pub package_source: PackageSource,
    pub fetch_date: String,
}

impl From<Package> for PackageResponseSchema {
    fn from(e: Package) -> Self {
        Self {
            id: e.id.unwrap_or_default(),
            package_id: e.package_id,
            version: e.version,
            package_source: e.package_source,
            fetch_date: e.fetch_date,
        }
    }
}
