use sqlx::{Pool, Sqlite};

use crate::features::package_service::{
    errors::PackageFetcherError,
    models::Package,
    schemas::{GetPackageSchema, InsertPackageSchema},
};

pub async fn insert_package(
    pool: &Pool<Sqlite>,
    schema: &InsertPackageSchema,
) -> Result<(), PackageFetcherError> {
    sqlx::query!(
        "INSERT INTO packages (cache_path ,package_id, version, package_source, fetch_date, size_bytes)
        VALUES (?, ?, ?, ?, ?, ?)",
        schema.cache_path,
        schema.package_id,
        schema.version,
        schema.package_source,
        schema.fetch_date,
        schema.size_bytes
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_package(
    pool: &Pool<Sqlite>,
    schema: &GetPackageSchema,
) -> Result<Option<Package>, PackageFetcherError> {
    let package = sqlx::query_as!(
        Package,
        "SELECT * FROM packages WHERE package_id = ? AND version = ? AND package_source = ?",
        schema.package_id,
        schema.version,
        schema.package_source
    )
    .fetch_optional(pool)
    .await?;

    Ok(package)
}

pub async fn get_first_in_package(
    pool: &Pool<Sqlite>,
) -> Result<Option<Package>, PackageFetcherError> {
    let package = sqlx::query_as!(Package, "SELECT * FROM packages ORDER BY id ASC LIMIT 1",)
        .fetch_optional(pool)
        .await?;

    Ok(package)
}

pub async fn get_total_bytes(pool: &Pool<Sqlite>) -> Result<i64, PackageFetcherError> {
    let bytes = sqlx::query_scalar!("SELECT COALESCE(SUM(size_bytes), 0) FROM packages")
        .fetch_one(pool)
        .await?;

    Ok(bytes)
}

pub async fn delete_package(pool: &Pool<Sqlite>, id: &i64) -> Result<(), PackageFetcherError> {
    sqlx::query!("DELETE FROM packages WHERE id = ?", id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_all_available_packages(pool: &Pool<Sqlite>) -> Result<Vec<Package>, PackageFetcherError>{
    let packages = sqlx::query_as!(
        Package,
        "SELECT * FROM packages"
    )
    .fetch_all(pool)
    .await?;

    Ok(packages)
}
