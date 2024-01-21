use chrono::NaiveDate;
use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use validator::Validate;
use crate::web::utils::validate_amount;

#[derive(Debug, Validate, Serialize, Clone, FromRow, Deserialize)]
pub struct Servicer {
    pub servicer_name: String,
    pub contact_name: String,
    pub contact_phone: String,
}