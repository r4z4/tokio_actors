use crate::web::utils::validate_amount;
use chrono::NaiveDate;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

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
    pub issue_month: String,
    pub loan_status: LoanStatus,
    pub initial_listing_status: InitialListingStatus,
    pub disbursement_method: DisbursementMethod,
    pub balance: f32,
    pub paid_total: f32,
    pub paid_principal: f32,
    pub paid_interest: f32,
    pub paid_late_fees: f32,
}

pub struct MockBalance {
    pub loan_amount: i32,
    pub installment: f32,
    pub term: i32,
    pub balance: f32,
    pub paid_total: f32,
    pub paid_principal: f32,
    pub paid_interest: f32,
    pub paid_late_fees: f32,
}

fn mock_balance() -> MockBalance {
    let amts = [80000, 75000, 55000, 75000];
    let terms = [12, 24, 48, 76, 96];
    let amt = amts[rand::thread_rng().gen_range(0..amts.len())];
    let pcts = [0.1_f32, 0.15, 0.2, 0.3];
    let term = terms[rand::thread_rng().gen_range(0..terms.len())];
    let inst = amt as f32 / term as f32;
    let installment = (inst * 100.0).round() / 100.0;
    let random_int: i32 = rand::thread_rng().gen_range(0..4);
    let payment_amt = random_int as f32 * installment;
    let balance = amt as f32 - payment_amt;
    let pct: f32 = pcts[rand::thread_rng().gen_range(0..pcts.len())];
    let paid_interest = payment_amt * pct;
    MockBalance {
        loan_amount: amt,
        term: term,
        installment: installment,
        balance: balance,
        paid_total: payment_amt,
        paid_principal: payment_amt - paid_interest,
        paid_interest: paid_interest,
        paid_late_fees: 0.0,
    }
}

pub fn mock_loan() -> Loan {
    let mut rng = rand::thread_rng();
    let emp_titles = ["President", "Sales Rep", "CEO", "Support Specialist"];
    let initial_listing_status = [
        InitialListingStatus::Whole,
        InitialListingStatus::Fractional,
    ];
    let disbursement_method = [DisbursementMethod::Cash, DisbursementMethod::DirectPay];
    let application_type = [ApplicationType::Joint, ApplicationType::Individual];
    let loan_purpose = [
        LoanPurpose::Moving,
        LoanPurpose::DebtConsolidation,
        LoanPurpose::Car,
        LoanPurpose::CreditCard,
        LoanPurpose::Other,
        LoanPurpose::HomeImprovement,
        LoanPurpose::MajorPurchase,
        LoanPurpose::House,
        LoanPurpose::Medical,
        LoanPurpose::Vacation,
        LoanPurpose::SmallBusiness,
    ];
    let loan_status = [
        LoanStatus::Current,
        LoanStatus::ChargedOff,
        LoanStatus::FullyPaid,
        LoanStatus::InGracePeriod,
        LoanStatus::Late1to15,
        LoanStatus::Late16to30,
        LoanStatus::Late31to120,
    ];
    let amts = [80000, 75000, 55000, 75000];
    let pd_amts = [830.23, 7110.22, 5220.12, 7330.11];
    let terms = [12, 24, 48, 76, 96];
    let floats: [f32; 3] = [4.4, 8.8, 33.3];
    let years = [1999, 2003, 2008, 2016, 2019];
    let one_to_five = rand::thread_rng().gen_range(0..5);
    let mock_balance = mock_balance();
    Loan {
        loan_purpose: loan_purpose[rand::thread_rng().gen_range(0..loan_purpose.len())].clone(),
        application_type: application_type[rand::thread_rng().gen_range(0..application_type.len())]
            .clone(),
        loan_amount: mock_balance.loan_amount,
        term: mock_balance.term,
        interest_rate: floats[rand::thread_rng().gen_range(0..floats.len())],
        installment: mock_balance.installment,
        grade: one_to_five,
        sub_grade: one_to_five,
        issue_month: "Apr-2022".to_owned(),
        loan_status: loan_status[rand::thread_rng().gen_range(0..loan_status.len())].clone(),
        initial_listing_status: initial_listing_status
            [rand::thread_rng().gen_range(0..initial_listing_status.len())]
        .clone(),
        disbursement_method: disbursement_method
            [rand::thread_rng().gen_range(0..disbursement_method.len())]
        .clone(),
        balance: mock_balance.balance,
        paid_total: pd_amts[rand::thread_rng().gen_range(0..pd_amts.len())],
        paid_principal: pd_amts[rand::thread_rng().gen_range(0..pd_amts.len())],
        paid_interest: pd_amts[rand::thread_rng().gen_range(0..pd_amts.len())],
        paid_late_fees: 0.0,
    }
}
