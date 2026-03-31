use std::path::PathBuf;

use chrono::Utc;
use sqlx::{Pool, Sqlite};

use crate::features::{
    disk_cache::cache::DiskCache,
    package_service::{
        errors::PackageFetcherError, helpers::calculate_bytes, repository, schemas::{GetPackageSchema, InsertPackageSchema}
    },
    zip_source::{GitHubSource, NpmSource, NuGetSource},
};

const MAX_BYTES: i64 = 5 * 1024 * 1024 * 1024;

pub async fn get_or_fetch_source(
    pool: &Pool<Sqlite>,
    cache: &DiskCache,
    req: &GetPackageSchema,
) -> Result<PathBuf, PackageFetcherError> {
    let package = repository::get_package(pool, req).await?;

    match package {
        Some(p) => return Ok(PathBuf::from(p.cache_path)),
        None => {
            let (_temp, source_dir) = match req.package_source {
                crate::features::sources::PackageSource::Github => {
                    let source = GitHubSource::new()?;
                    source
                        .download_and_extract(&req.package_id, &req.version)
                        .await?
                }
                crate::features::sources::PackageSource::Nuget => {
                    let source = NuGetSource::new()?;
                    source
                        .download_and_extract(&req.package_id, &req.version)
                        .await?
                }
                crate::features::sources::PackageSource::Npm => {
                    let source = NpmSource::new()?;
                    source
                        .download_and_extract(&req.package_id, &req.version)
                        .await?
                }
            };

            let cache_key = format!(
                "{:?}/{}-{}",
                req.package_source, &req.package_id, &req.version
            );

            let cache_path = cache.get_path(&cache_key).await?;

            cache.insert(&cache_key, &source_dir).await?;

            drop(_temp);

            repository::insert_package(
                pool,
                &InsertPackageSchema {
                    cache_path: cache_path.to_string_lossy().to_string(),
                    package_id: req.package_id.clone(),
                    version: req.version.clone(),
                    package_source: req.package_source.clone(),
                    fetch_date: Utc::now(),
                    size_bytes: calculate_bytes(&cache_path).await as i64,
                },
            )
            .await?;

            evict_by_size(pool, cache).await?;

            return Ok(cache_path);
        }
    }
}

pub async fn delete_source(
    pool: &Pool<Sqlite>,
    cache: &DiskCache,
    id: i64,
) -> Result<(), PackageFetcherError> {
    if let Some(p) = repository::get_package_by_id(pool, &id).await? {
        cache.delete_dir(PathBuf::from(&p.cache_path).as_path()).await?;
    }
    repository::delete_package(pool, &id).await?;
    Ok(())
}

async fn evict_by_size(pool: &Pool<Sqlite>, cache: &DiskCache) -> Result<(), PackageFetcherError> {
    tracing::info!("checking db against max_bytes");
    let mut total_bytes = repository::get_total_bytes(pool).await?;
    while total_bytes > MAX_BYTES {
        tracing::info!("Max bytes reached, cleaning...");

        match repository::get_first_in_package(pool).await? {
            Some(p) => {
                cache
                    .delete_dir(PathBuf::from(&p.cache_path).as_path())
                    .await?;
                repository::delete_package(pool, &p.id.unwrap_or_default()).await?;
                total_bytes -= p.size_bytes;
            }
            None => return Ok(()),
        }
    }

    Ok(())
}
