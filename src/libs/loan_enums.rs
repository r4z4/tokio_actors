use std::str::FromStr;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::models::loan::{LoanPurpose, LoanStatus};

#[inline]
pub fn remove_prefix(date_str: &str) -> &str {
    date_str.trim_start_matches("Reviewed ")
}

fn date_str_into_parts(date_str: &str) -> Option<(&str, &str, &str)> {
    let mut parts = date_str.split_whitespace();
    let month = parts.next()?;
    let day = parts.next()?;
    let year = parts.next()?;
    Some((month, day, year))
}

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

impl std::str::FromStr for LoanPurpose {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "moving" => Ok(LoanPurpose::Moving),
            "medical" => Ok(LoanPurpose::Medical),
            "debt consolidation" => Ok(LoanPurpose::DebtConsolidation),
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
            _ => Err("Invalid LoanStatus value"),
        }
    }
}

pub fn convert_loan_purpose(lp_string: &str) -> Result<LoanPurpose, &'static str> {
    LoanPurpose::from_str(lp_string)
}

pub fn convert_loan_status(ls_string: &str) -> Result<LoanStatus, &'static str> {
    LoanStatus::from_str(ls_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_loan_purpose() {
        const LP: &str = "medical";
        let converted_lp = convert_loan_purpose(LP).unwrap();
        assert_eq!(converted_lp, LoanPurpose::Medical);
    }
    #[test]
    fn test_convert_loan_status() {
        const LS: &str = "In Grace Period";
        let converted_ls = convert_loan_status(LS).unwrap();
        assert_eq!(converted_ls, LoanStatus::InGracePeriod);
    }
}