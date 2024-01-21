use sqlx::{postgres::PgPoolOptions, Postgres, Pool};

use crate::{error::AppError, config::config};

pub type Db = Pool<Postgres>;

pub async fn new_db_pool() -> Result<Db, AppError> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&config().DB_URL)
        .await.map_err(|ex| AppError::FailToCreatePool)
}