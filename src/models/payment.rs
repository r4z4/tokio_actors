use crate::web::utils::validate_amount;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Validate, Serialize, Clone, Deserialize)]
pub struct CreditCardApiResp {
    pub id: i32,
    pub uid: String,
    pub credit_card_number: String,
    pub credit_card_expiry_date: String,
    pub credit_card_type: String,
}

#[derive(Debug, Validate, Serialize, Clone, FromRow, Deserialize)]
pub struct CreditCard {
    pub id: i32,
    pub uid: String,
    pub credit_card_number: String,
    pub credit_card_expiry_date: NaiveDate,
    pub credit_card_type: String,
}

#[derive(Debug, Validate, Serialize, Clone, Deserialize)]
pub struct BankApiResp {
    pub id: i32,
    pub uid: String,
    pub account_number: String,
    pub iban: String,
    pub bank_name: String,
    pub routing_number: String,
    pub swift_bic: String,
}
