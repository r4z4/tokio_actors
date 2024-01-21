use axum::{Extension, Json};
use serde_json::{Value, json};
use sqlx::PgPool;

use crate::{error::AppError, models};

pub async fn register(
    Json(credentials): Json<models::auth::LoginUser>,
    Extension(pool): Extension<PgPool>,
) -> Result<Json<Value>, AppError> {
    if credentials.email.is_empty() || credentials.password.is_empty() {
        return Err(AppError::MissingCredential("test".to_owned()));
    }
    let user = sqlx::query_as::<_, models::auth::User>(
        "SELECT email, password FROM users WHERE email = $1"
    )
    .bind(&credentials.email)
    .fetch_optional(&pool)
    .await
    .map_err(|err| {
        dbg!(err);
        AppError::InternalServerError
    })?;

    if let Some(_) = user {
        return Err(AppError::UserAlreadyExists("test".to_owned()));
    }

    let result = sqlx::query("INSERT INTO users (email, password) VALUES ($1,$2)")
        .bind(&credentials.email)
        .bind(&credentials.password)
        .execute(&pool)
        .await
        .map_err(|_| AppError::InternalServerError)?;
    if result.rows_affected() < 1 {
        Err(AppError::InternalServerError)
    } else {
        Ok(Json(json!({ "msg": "registered successfully" })))
    }

}