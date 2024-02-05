use chrono::NaiveDate;
use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use validator::Validate;
use crate::web::utils::validate_amount;

#[derive(Debug, Validate, Serialize, Clone, FromRow, Deserialize)]
pub struct Offer {
    pub offer_slug: String,
    pub servicer_id: i32,
    pub max_amount: i32,
    #[validate(custom = "validate_amount")]
    pub min_amount: i32,
    pub terms: i32,
    pub percent_fee: f32,
    pub apr: f32,
    pub expires: NaiveDate,
}