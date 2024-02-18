use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::models::loan::{
    ApplicationType, DisbursementMethod, InitialListingStatus, LoanPurpose, LoanStatus,
};

impl Into<u32> for LoanPurpose {
    fn into(self) -> u32 {
        self as u32
    }
}

impl Into<u32> for LoanStatus {
    fn into(self) -> u32 {
        self as u32
    }
}

impl Into<u32> for DisbursementMethod {
    fn into(self) -> u32 {
        self as u32
    }
}

impl Into<u32> for ApplicationType {
    fn into(self) -> u32 {
        self as u32
    }
}

impl Into<u32> for InitialListingStatus {
    fn into(self) -> u32 {
        self as u32
    }
}

impl std::str::FromStr for LoanPurpose {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "moving" => Ok(LoanPurpose::Moving),
            "medical" => Ok(LoanPurpose::Medical),
            "debt_consolidation" => Ok(LoanPurpose::DebtConsolidation),
            "credit_card" => Ok(LoanPurpose::CreditCard),
            "home_improvement" => Ok(LoanPurpose::HomeImprovement),
            "car" => Ok(LoanPurpose::Car),
            "house" => Ok(LoanPurpose::House),
            "major_purchase" => Ok(LoanPurpose::MajorPurchase),
            "vacation" => Ok(LoanPurpose::Vacation),
            "small_business" => Ok(LoanPurpose::SmallBusiness),
            "other" => Ok(LoanPurpose::Other),
            _ => Err("Invalid LoanPurpose value"),
        }
    }
}

impl std::str::FromStr for LoanStatus {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Current" => Ok(LoanStatus::Current),
            "Fully Paid" => Ok(LoanStatus::FullyPaid),
            "In Grace Period" => Ok(LoanStatus::InGracePeriod),
            "Late (1-15 days)" => Ok(LoanStatus::Late1to15),
            "Late (16-30 days)" => Ok(LoanStatus::Late16to30),
            "Late (31-120 days)" => Ok(LoanStatus::Late31to120),
            "Charged Off" => Ok(LoanStatus::ChargedOff),
            _ => Err("Invalid LoanStatus value"),
        }
    }
}

impl std::str::FromStr for DisbursementMethod {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Cash" => Ok(DisbursementMethod::Cash),
            "DirectPay" => Ok(DisbursementMethod::DirectPay),
            _ => Err("Invalid DisbursementMethod value"),
        }
    }
}

impl std::str::FromStr for ApplicationType {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Individual" => Ok(ApplicationType::Individual),
            "Joint" => Ok(ApplicationType::Joint),
            _ => Err("Invalid ApplicationType value"),
        }
    }
}

impl std::str::FromStr for InitialListingStatus {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        dbg!(s);
        match s {
            "whole" => Ok(InitialListingStatus::Whole),
            "fractional" => Ok(InitialListingStatus::Fractional),
            e => Err("Invalid InitialListingStatus value {e}"),
        }
    }
}

pub fn convert_loan_purpose(lp_str: &str) -> Result<LoanPurpose, &'static str> {
    LoanPurpose::from_str(lp_str)
}

pub fn convert_loan_status(ls_str: &str) -> Result<LoanStatus, &'static str> {
    LoanStatus::from_str(ls_str)
}

pub fn convert_disbursement_method(dm_str: &str) -> Result<DisbursementMethod, &'static str> {
    DisbursementMethod::from_str(dm_str)
}

pub fn convert_application_type(at_str: &str) -> Result<ApplicationType, &'static str> {
    ApplicationType::from_str(at_str)
}

pub fn convert_initial_listing_status(ils_str: &str) -> Result<InitialListingStatus, &'static str> {
    InitialListingStatus::from_str(ils_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_loan_purpose() {
        const TEST_STR: &str = "medical";
        let converted_str = convert_loan_purpose(TEST_STR).unwrap();
        assert_eq!(converted_str, LoanPurpose::Medical);
    }
    #[test]
    fn test_convert_loan_status() {
        const TEST_STR: &str = "In Grace Period";
        let converted_str = convert_loan_status(TEST_STR).unwrap();
        assert_eq!(converted_str, LoanStatus::InGracePeriod);
    }

    #[test]
    fn test_convert_disbursement_method() {
        const TEST_STR: &str = "DirectPay";
        let converted_str = convert_disbursement_method(TEST_STR).unwrap();
        assert_eq!(converted_str, DisbursementMethod::DirectPay);
    }

    #[test]
    fn test_convert_application_type() {
        const TEST_STR: &str = "Individual";
        let converted_str = convert_application_type(TEST_STR).unwrap();
        assert_eq!(converted_str, ApplicationType::Individual);
    }

    #[test]
    fn test_convert_initial_listing_status() {
        const TEST_STR: &str = "fractional";
        let converted_str = convert_initial_listing_status(TEST_STR).unwrap();
        assert_eq!(converted_str, InitialListingStatus::Fractional);
    }
}
