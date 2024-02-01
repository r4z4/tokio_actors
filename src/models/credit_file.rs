use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Validate, Serialize, Clone, FromRow, Deserialize)]
pub struct CreditFile {
    pub borrower_id: i32,
    pub emp_title: String,
    pub emp_length: i32,
    pub state: String,
    pub homeownership: HomeOwnership,
    pub annual_income: f32,
    pub verified_income: IncomeVerification,
    pub debt_to_income: f32,
    pub annual_income_joint: Option<i32>,
    pub verification_income_joint: Option<IncomeVerification>,
    pub debt_to_income_joint: Option<f32>,
    pub delinq_2y: i32,
    pub months_since_last_delinq: i32,
    pub earliest_credit_line: i32,
    pub inquiries_last_12m: i32,
    pub total_credit_lines: i32,
    pub open_credit_lines: i32,
    pub total_credit_limit: i32,
    pub total_credit_utilized: i32,
    pub num_collections_last_12m: f32,
    pub num_historical_failed_to_pay: f32,
    pub months_since_90d_late: i32,
    pub current_accounts_delinq: i32,	
    pub total_collection_amount_ever: i32,	
    pub current_installment_accounts: i32,	
    pub accounts_opened_24m: i32,
    pub months_since_last_credit_inquiry: i32,	
    pub num_satisfactory_accounts: i32,
    pub num_accounts_120d_past_due: i32,
    pub num_accounts_30d_past_due: i32,
    pub num_active_debit_accounts: i32,
    pub total_debit_limit: i32,
    pub num_total_cc_accounts: i32,
    pub num_open_cc_accounts: i32,
    pub num_cc_carrying_balance: i32,
    pub num_mort_accounts: i32,
    pub account_never_delinq_percent: f32,
    // Tell Serde CSV col name if we want a diff name in struct
    // #[serde(rename = "num_historical_failed_to_pay")]
    pub tax_liens: i32,
    pub public_record_bankrupt: i32,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[repr(u32)]
pub enum HomeOwnership {
    Own = 1,
    Mortgage = 2,
    Rent = 3,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[repr(u32)]
pub enum IncomeVerification {
    Verified = 1,
    SourceVerified = 2,
    NotVerified = 3,
}