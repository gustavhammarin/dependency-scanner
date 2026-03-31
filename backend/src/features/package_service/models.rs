use serde::{Deserialize, Serialize};

use crate::features::sources::PackageSource;

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct Package{
    pub id: Option<i64>,
    pub cache_path: String,
    pub package_id: String,
    pub version: String,
    pub package_source: PackageSource,
    pub fetch_date: String,
    pub size_bytes: i64
}
