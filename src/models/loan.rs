use chrono::NaiveDate;
use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use validator::Validate;
use crate::web::utils::validate_amount;

pub enum LoanStatus {
    Current,
    FullyPaid,
    InGracePeriod
}

pub enum LoanPurpose {
    Moving,
    Medical,
    DebtConsolidation
}

pub enum DisbursementMethod {
    Cash,
    DirectPay
}

pub enum ApplicationType {
    Individual,
    Joint
}

pub enum InitialListingStatus {
    Whole,
    Fractional
}

#[derive(Debug, Validate, Serialize, Clone, FromRow, Deserialize)]
pub struct Loan {
    pub emp_title: i32,
    pub emp_length: i32,
    #[validate(custom = "validate_amount")]
    pub state: i32,
    pub homeownership: i32,
    pub annual_income: f32,
    pub verified_income: f32,
    pub debt_to_income: NaiveDate,
    pub annual_income_joint: i32,
    pub verification_income_joint: i32,
    pub debt_to_income_joint: i32,
    pub total_credit_lines: i32,
    pub open_credit_lines: i32,
    pub total_credit_limit: i32,
    pub total_credit_utilized: i32,
    pub num_collections_last_12m: f32,
    pub num_historical_failed_to_pay: f32,
    pub num_total_cc_accounts: i32,
    pub num_open_cc_accounts: i32,
    pub tax_liens: i32,
    pub public_record_bankrupt: i32,
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
