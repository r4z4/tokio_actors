use std::time::Duration;

use askama::Template;
use axum::{Extension, Json};
use serde_json::{json, Value};
use sqlx::PgPool;

use crate::{config::ApiError, error::AppError, models};

pub async fn task_dump() -> Result<String, ApiError> {
    let handle = tokio::runtime::Handle::current();

    if let Ok(dump) = tokio::time::timeout(Duration::from_secs(2), handle.dump()).await {
        let mut dump_str = String::new();
        for (i, task) in dump.tasks().iter().enumerate() {
            let trace = task.trace();
            let trace_str = "task {i}: {trace}\n";
            dump_str.push_str(trace_str);
            // println!("task {i}:");
            // println!("{trace}\n");
        }
        Ok(dump_str)
    } else {
        Err(ApiError::from(("Dump error", 1)))
    }
}