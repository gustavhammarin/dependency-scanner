use axum::{Json, extract::State};

use crate::{
    AppState,
    features::dps_scan::{
        errors::ScanError,
        models::{DpsScanRequest, DpsScanResult},
        service,
    },
};

pub async fn dps_scan_handler(
    State(state): State<AppState>,
    Json(req): Json<DpsScanRequest>,
) -> Result<Json<DpsScanResult>, ScanError> {
    let result = service::run_scanning(
        &state.db,
        &state.cache,
        &req.source,
        &req.package_id,
        &req.package_version,
    )
    .await?;
    Ok(Json(result))
}
