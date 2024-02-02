use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

use crate::libs::credit_file_enums::{deserialize_homeownership, deserialize_income_verification, deserialize_na_col};

#[derive(Debug, Validate, Serialize, Clone, FromRow, Deserialize)]
pub struct CreditFile {
    pub borrower_id: i32,
    pub emp_title: Option<String>,
    pub emp_length: Option<i32>,
    pub state: String,
    #[serde(deserialize_with = "deserialize_homeownership")]
    pub homeownership: HomeOwnership,
    pub annual_income: i32,
    #[serde(deserialize_with = "deserialize_income_verification")]
    pub verified_income: IncomeVerification,
    pub debt_to_income: f32,
    pub annual_income_joint: Option<i32>,
    #[serde(deserialize_with = "deserialize_income_verification")]
    pub verification_income_joint: IncomeVerification,
    pub debt_to_income_joint: Option<f32>,
    pub delinq_2y: i32,
    // Cols with "NA" or Integer need to handled
    #[serde(deserialize_with = "deserialize_na_col")]
    pub months_since_last_delinq: Option<i32>,
    pub earliest_credit_line: i32,
    pub inquiries_last_12m: i32,
    pub total_credit_lines: Option<i32>,
    pub open_credit_lines: i32,
    pub total_credit_limit: i32,
    pub total_credit_utilized: i32,
    pub num_collections_last_12m: i32,
    pub num_historical_failed_to_pay: i32,
    pub months_since_90d_late: Option<i32>,
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

pub fn mock_credit_file() -> CreditFile {
    let mut rng = rand::thread_rng();
    static HOMES: [HomeOwnership; 3] = [HomeOwnership::Own, HomeOwnership::Rent, HomeOwnership::Mortgage];
    let emp_titles = ["President", "Sales Rep", "CEO", "Support Specialist"];
    let income_verification = [IncomeVerification::Verified,IncomeVerification::NotVerified, IncomeVerification::SourceVerified];
    let homeownership = [HomeOwnership::Own,HomeOwnership::Mortgage, HomeOwnership::Rent];
    let incomes = [80000,75000,55000,75000];
    let limits = [84756, 37463, 57343, 2234, 47364];
    let floats: [f32; 3] = [4.4, 8.8, 33.3];
    let years = [1999, 2003, 2008, 2016, 2019];
    let one_to_five = rand::thread_rng().gen_range(0..5);
    CreditFile {
        borrower_id: 2,
        emp_title: Some(emp_titles[rand::thread_rng().gen_range(0..emp_titles.len())].to_owned()),
        emp_length: Some(one_to_five),
        state: "MN".to_owned(),
        homeownership: homeownership[rand::thread_rng().gen_range(0..homeownership.len())].clone(),
        annual_income: incomes[rand::thread_rng().gen_range(0..incomes.len())],
        verified_income: income_verification[rand::thread_rng().gen_range(0..income_verification.len())].clone(),
        debt_to_income: floats[rand::thread_rng().gen_range(0..floats.len())],
        annual_income_joint: Some(incomes[rand::thread_rng().gen_range(0..incomes.len())]),
        verification_income_joint: income_verification[rand::thread_rng().gen_range(0..income_verification.len())].clone(),
        debt_to_income_joint: Some(floats[rand::thread_rng().gen_range(0..floats.len())]),
        delinq_2y: one_to_five,
        months_since_last_delinq: Some(one_to_five),
        earliest_credit_line: years[rand::thread_rng().gen_range(0..years.len())],
        inquiries_last_12m: one_to_five,
        total_credit_lines: Some(one_to_five),
        open_credit_lines: one_to_five,
        total_credit_limit: one_to_five,
        total_credit_utilized: one_to_five,
        num_collections_last_12m: one_to_five,
        num_historical_failed_to_pay: one_to_five,
        months_since_90d_late: Some(one_to_five),
        current_accounts_delinq: one_to_five,	
        total_collection_amount_ever: one_to_five,	
        current_installment_accounts: one_to_five,	
        accounts_opened_24m: one_to_five,
        months_since_last_credit_inquiry: one_to_five,	
        num_satisfactory_accounts: one_to_five,
        num_accounts_120d_past_due: one_to_five,
        num_accounts_30d_past_due: one_to_five,
        num_active_debit_accounts: one_to_five,
        total_debit_limit: incomes[rand::thread_rng().gen_range(0..incomes.len())],
        num_total_cc_accounts: one_to_five,
        num_open_cc_accounts: one_to_five,
        num_cc_carrying_balance: one_to_five,
        num_mort_accounts: one_to_five,
        account_never_delinq_percent: 100.00,
        tax_liens: 0,
        public_record_bankrupt: 0,
    }
}