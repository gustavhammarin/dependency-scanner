use std::{env, sync::Arc, time::Duration};

use dotenvy::dotenv;
use sqlx::SqlitePool;

use crate::{app::app, db::create_pool, features::disk_cache::cache::DiskCache};

mod app;
mod db;
mod features;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub cache: Arc<DiskCache>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    tracing_subscriber::fmt::init();

    std::panic::set_hook(Box::new(|panic_info| {
        tracing::error!("PANIC: {}", panic_info);
    }));

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:data.db".to_string());

    let db = create_pool(&database_url).await?;
    let cache = DiskCache::new("dependency_scanner", Duration::from_hours(2), Duration::from_mins(30)).await?;

    let state = AppState { db, cache: Arc::new(cache) };

    let app = app(state);

    let port = env::var("RUST_PORT").unwrap_or_else(|_| "5000".to_string());
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("Starting server on {addr}");

    axum::serve(listener, app).await?;

    Ok(())
}
