use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Validate, Serialize, FromRow, Clone, Deserialize)]
pub struct SelectOption {
    pub value: i32,
    pub key: String,
}