use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use struct_iterable::Iterable;
use validator::Validate;

use crate::libs::credit_file_enums::{
    deserialize_homeownership, deserialize_income_verification, deserialize_na_col,
};

#[derive(Debug, Validate, Serialize, Clone, FromRow, Deserialize, PartialEq, Iterable)]
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
    pub debt_to_income: Option<f32>,
    pub annual_income_joint: Option<i32>,
    #[serde(deserialize_with = "deserialize_income_verification")]
    pub verification_income_joint: IncomeVerification,
    pub debt_to_income_joint: Option<f32>,
    pub delinq_2y: i32,
    // Cols with "NA" or Integer need to handled
    // Just replaced them in CSV - This worked but had cols with NA and empty
    // #[serde(deserialize_with = "deserialize_na_col")]
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
    pub months_since_last_credit_inquiry: Option<i32>,
    pub num_satisfactory_accounts: i32,
    pub num_accounts_120d_past_due: Option<i32>,
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

impl CreditFile {
    fn hamming_distance(&self, other: &Self) -> usize {
        if other == self {
            // They all match
            other.iter().count()
        } else {
            let mut count = 0;

            // if self.borrower_id == other.borrower_id {count += 1} else {};
            if self.emp_title == other.emp_title {
                count += 1
            } else {
            };
            if self.emp_length == other.emp_length {
                count += 1
            } else {
            };
            if self.state == other.state {
                count += 1
            } else {
            };
            if self.homeownership == other.homeownership {
                count += 1
            } else {
            };
            if self.annual_income == other.borrower_id {
                count += 1
            } else {
            };

            // if self.verification_income_joint == other.verification_income_joint {count += 1} else {};
            // if self.debt_to_income == other.debt_to_income {count += 1} else {};
            // if self.delinq_2y == other.delinq_2y {count += 1} else {};
            // if self.months_since_last_delinq == other.months_since_last_delinq {count += 1} else {};
            // if self.earliest_credit_line == other.earliest_credit_line {count += 1} else {};
            // if self.inquiries_last_12m == other.inquiries_last_12m {count += 1} else {};
            // if self.total_credit_lines == other.total_credit_lines {count += 1} else {};
            // if self.open_credit_lines == other.open_credit_lines {count += 1} else {};
            // if self.total_credit_limit == other.total_credit_limit {count += 1} else {};
            // if self.total_credit_utilized == other.total_credit_utilized {count += 1} else {};
            // if self.num_collections_last_12m == other.num_collections_last_12m {count += 1} else {};
            // if self.num_historical_failed_to_pay == other.num_historical_failed_to_pay {count += 1} else {};
            // if self.months_since_90d_late == other.months_since_90d_late {count += 1} else {};
            // if self.current_accounts_delinq == other.current_accounts_delinq {count += 1} else {};
            // if self.total_collection_amount_ever == other.total_collection_amount_ever {count += 1} else {};
            // if self.current_installment_accounts == other.current_installment_accounts {count += 1} else {};
            // if self.accounts_opened_24m == other.accounts_opened_24m {count += 1} else {};
            // if self.months_since_last_credit_inquiry == other.months_since_last_credit_inquiry {count += 1} else {};
            // if self.num_satisfactory_accounts == other.num_satisfactory_accounts {count += 1} else {};
            // if self.num_accounts_120d_past_due == other.num_accounts_120d_past_due {count += 1} else {};
            // if self.num_accounts_30d_past_due == other.num_accounts_30d_past_due {count += 1} else {};
            // if self.num_active_debit_accounts == other.num_active_debit_accounts {count += 1} else {};
            // if self.total_debit_limit == other.total_debit_limit {count += 1} else {};
            // if self.num_total_cc_accounts == other.num_total_cc_accounts {count += 1} else {};
            // if self.num_open_cc_accounts == other.num_open_cc_accounts {count += 1} else {};
            // if self.num_cc_carrying_balance == other.num_cc_carrying_balance {count += 1} else {};
            // if self.num_mort_accounts == other.num_mort_accounts {count += 1} else {};
            // if self.account_never_delinq_percent == other.account_never_delinq_percent {count += 1} else {};
            // if self.tax_liens == other.tax_liens {count += 1} else {};
            // if self.public_record_bankrupt == other.public_record_bankrupt {count += 1} else {};

            count
        }
    }
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
    Empty = 0,
}

pub fn mock_credit_file() -> CreditFile {
    let mut rng = rand::thread_rng();
    static HOMES: [HomeOwnership; 3] = [
        HomeOwnership::Own,
        HomeOwnership::Rent,
        HomeOwnership::Mortgage,
    ];
    let emp_titles = ["President", "Sales Rep", "CEO", "Support Specialist"];
    let income_verification = [
        IncomeVerification::Verified,
        IncomeVerification::NotVerified,
        IncomeVerification::SourceVerified,
    ];
    let homeownership = [
        HomeOwnership::Own,
        HomeOwnership::Mortgage,
        HomeOwnership::Rent,
    ];
    let incomes = [80000, 75000, 55000, 75000];
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
        verified_income: income_verification
            [rand::thread_rng().gen_range(0..income_verification.len())]
        .clone(),
        debt_to_income: Some(floats[rand::thread_rng().gen_range(0..floats.len())]),
        annual_income_joint: Some(incomes[rand::thread_rng().gen_range(0..incomes.len())]),
        verification_income_joint: income_verification
            [rand::thread_rng().gen_range(0..income_verification.len())]
        .clone(),
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
        months_since_last_credit_inquiry: Some(one_to_five),
        num_satisfactory_accounts: one_to_five,
        num_accounts_120d_past_due: Some(one_to_five),
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn equal_records_returns_len() {
        let cf = mock_credit_file();
        let cf2 = cf.clone();
        let len = cf.iter().count();
        let dist = cf.hamming_distance(&cf2);
        assert_eq!(dist, len);
    }
    // This should almost always pass FIXME
    #[test]
    fn diff_records_returns_less_than_len() {
        let cf = mock_credit_file();
        let cf2 = mock_credit_file();
        let len = cf.iter().count();
        let dist = cf.hamming_distance(&cf2);
        assert!(dist < len);
    }
    // #[test]
    // fn test_date_convert() {
    //     let converted_date = convert_date(DATE_STR).unwrap();
    //     assert_eq!(converted_date, NaiveDate::from_ymd_opt(2023, 8, 25).unwrap());
    // }
}
