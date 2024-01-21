#![allow(unused)] // FIXME: Remove
use password_auth::generate_hash;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::web::App;
mod web;
mod users;
// Import modules
mod actors;
mod controllers;
mod error;
mod models;
mod config;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(EnvFilter::new(std::env::var("RUST_LOG").unwrap_or_else(
            |_| "axum_login=debug,tower_sessions=debug,sqlx=warn,tower_http=debug".into(),
        )))
        .with(tracing_subscriber::fmt::layer())
        .try_init()?;

    // tracing_subscriber::fmt()
    //     .with_target(false)
    //     // .compact()
    //     .json()
    //     .init();

    App::new().await?.serve().await
}