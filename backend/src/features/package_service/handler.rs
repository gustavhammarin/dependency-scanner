use axum::{Json, extract::{Path, State}};

use crate::{AppState, features::package_service::{errors::PackageFetcherError, repository, schemas::{GetPackageSchema, PackageResponseSchema}, service::{delete_source, get_or_fetch_source}}};

pub async fn get_all_packages_handler(
    State(state): State<AppState>
) -> Result<Json<Vec<PackageResponseSchema>>, PackageFetcherError>{
    let packages = repository::get_all_available_packages(&state.db).await?;
    Ok(Json(packages.into_iter().map(|e| e.into()).collect()))
}

pub async fn delete_package_handler(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<(), PackageFetcherError> {
    delete_source(&state.db, &state.cache, id).await
}

pub async fn fetch_new_source_handler(
    State(state): State<AppState>,
    Json(req): Json<GetPackageSchema>
) -> Result<Json<PackageResponseSchema>, PackageFetcherError>{
    get_or_fetch_source(&state.db, &state.cache, &req).await?;
    let package = repository::get_package(&state.db, &req)
        .await?
        .ok_or_else(|| PackageFetcherError::DbError(sqlx::Error::RowNotFound))?;
    Ok(Json(package.into()))
}
