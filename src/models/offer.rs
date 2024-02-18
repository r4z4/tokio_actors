use crate::web::utils::validate_amount;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

use super::auth::CurrentUser;

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

// Ensure can be sent safely between thread
// Don't even need to run test. Compile time check.
fn is_normal<T: Sized + Send + Sync + Unpin>() {}

#[test]
fn normal_types() {
    is_normal::<Offer>();
}
