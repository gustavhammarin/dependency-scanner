use axum::{
    Router,
    http::HeaderValue,
    routing::{delete, get, post},
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::{
    AppState,
    features::{
        dps_scan::handler::dps_scan_handler,
        package_service,
    },
};

pub fn app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin([
            "http://localhost:5173".parse::<HeaderValue>().unwrap(),
            "http://localhost:3000".parse::<HeaderValue>().unwrap(),
        ])
        .allow_headers(Any)
        .allow_methods(Any);

    let api = Router::new()
        .route("/dependency-scan", post(dps_scan_handler))
        .route("/packages", get(package_service::handler::get_all_packages_handler))
        .route("/packages", post(package_service::handler::fetch_new_source_handler))
        .route("/packages/{id}", delete(package_service::handler::delete_package_handler));

    Router::new()
        .nest("/api", api)
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}
