use crate::web::utils::validate_amount;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Validate, Serialize, Clone, FromRow, Deserialize)]
pub struct Servicer {
    pub servicer_name: String,
    pub contact_name: String,
    pub contact_phone: String,
}
