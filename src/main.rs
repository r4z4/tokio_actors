#![allow(unused)] use std::{collections::HashSet, env, sync::Mutex};

// FIXME: Remove
use password_auth::generate_hash;
use tokio::sync::broadcast;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use sqlx::{Pool, Postgres, QueryBuilder};
use crate::web::App;
mod users;
mod web;
// Import modules
mod actors;
mod config;
mod controllers;
mod error;
mod libs;
mod models;
mod redis_mod;

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    dbg!(&args);
    // The program name takes up the first arg args[0]
    let run_migration =
        if args.len() > 1 {
            &args[1]
        } else {
            let resp = "";
            resp
        };
    dbg!(&run_migration);
    // Perform some initial migrations
    // FIXME: Move to Refinery SQL which allows .rs migrations

    let console_layer = console_subscriber::ConsoleLayer::builder().spawn();
    tracing_subscriber::registry()
        .with(console_layer)
        .with(EnvFilter::new(std::env::var("RUST_LOG").unwrap_or_else(
            |_| "axum_login=debug,tower_sessions=debug,sqlx=warn,tower_http=debug,tokio=trace,runtime=trace".into(),
        )))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // tracing_subscriber::fmt()
    //     .with_target(false)
    //     // .compact()
    //     .json()
    //     .init();

    App::new(run_migration).await?.serve().await
}
