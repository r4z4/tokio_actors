use chrono::NaiveDate;
use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use validator::Validate;
use crate::web::utils::validate_amount;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[repr(u32)]
pub enum LoanStatus {
    Current = 1,
    FullyPaid = 2,
    InGracePeriod = 3,
    Late1to15 = 4,
    Late16to30 = 5,
    Late31to120 = 6,
    ChargedOff = 7,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[repr(u32)]
pub enum LoanPurpose {
    Moving = 1,
    Medical = 2,
    DebtConsolidation = 3,
    CreditCard = 4,
    HomeImprovement = 5,
    Car = 6,
    House = 7,
    MajorPurchase = 8,
    Vacation = 9,
    SmallBusiness = 10,
    Other = 11,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[repr(u32)]
pub enum DisbursementMethod {
    Cash = 1,
    DirectPay = 2,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[repr(u32)]
pub enum ApplicationType {
    Individual = 1,
    Joint = 2,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[repr(u32)]
pub enum InitialListingStatus {
    Whole = 1,
    Fractional = 2,
}

#[derive(Debug, Validate, Serialize, Clone, FromRow, Deserialize)]
pub struct Loan {
    pub loan_purpose: LoanPurpose,
    pub application_type: ApplicationType,
    pub loan_amount: i32,
    pub term: i32,
    pub interest_rate: f32,
    pub installment: f32,
    pub grade: i32,
    pub sub_grade: i32,
    pub issue_month: f32,
    pub loan_status: LoanStatus,
    pub initial_listing_status: InitialListingStatus,
    pub disbursement_method: DisbursementMethod,
    pub balance: f32,
    pub paid_total: f32,
    pub paid_principal: f32,
    pub paid_interest: f32,
    pub paid_late_fees: f32,
}
