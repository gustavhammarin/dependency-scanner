use sqlx::{Pool, Sqlite};

use crate::{
    features::disk_cache::cache::DiskCache,
    features::{
        dps_scan::{
            errors::ScanError,
            models::DpsScanResult,
            scanner_pipeline,
        },
        package_service::{schemas::GetPackageSchema, service::get_or_fetch_source},
        sources::PackageSource,
    },
};

pub async fn run_scanning(
    pool: &Pool<Sqlite>,
    cache: &DiskCache,
    package_source: &PackageSource,
    package_id: &str,
    package_version: &str,
) -> Result<DpsScanResult, ScanError> {
    let source_dir = get_or_fetch_source(
        pool,
        cache,
        &GetPackageSchema {
            package_id: package_id.to_string(),
            version: package_version.to_string(),
            package_source: package_source.to_owned(),
        },
    )
    .await?;

    scanner_pipeline::scan_directory(&source_dir).await
}
