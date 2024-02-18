use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::models::credit_file::{HomeOwnership, IncomeVerification};

impl Into<u32> for HomeOwnership {
    fn into(self) -> u32 {
        self as u32
    }
}

impl Into<u32> for IncomeVerification {
    fn into(self) -> u32 {
        self as u32
    }
}

// Appears in the messy CSV as 'Jan.'
impl std::str::FromStr for HomeOwnership {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "OWN" => Ok(HomeOwnership::Own),
            "MORTGAGE" => Ok(HomeOwnership::Mortgage),
            "RENT" => Ok(HomeOwnership::Rent),
            _ => Err("Invalid HomeOwnership value"),
        }
    }
}

impl std::str::FromStr for IncomeVerification {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Verified" => Ok(IncomeVerification::Verified),
            "Source Verified" => Ok(IncomeVerification::SourceVerified),
            "Not Verified" => Ok(IncomeVerification::NotVerified),
            _ => Ok(IncomeVerification::Empty),
        }
    }
}

pub fn convert_homeownership(ho_str: &str) -> Result<HomeOwnership, &'static str> {
    HomeOwnership::from_str(ho_str)
}

pub fn convert_income_verification(iv_str: &str) -> Result<IncomeVerification, &'static str> {
    IncomeVerification::from_str(iv_str)
}

pub fn handle_na_col(col_val: &str) -> Result<Option<i32>, &'static str> {
    match col_val {
        "NA" => Ok(None),
        _ => Ok(Some(col_val.parse::<i32>().unwrap())),
    }
}

pub fn deserialize_homeownership<'de, D>(deserializer: D) -> Result<HomeOwnership, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let ho_str: &str = Deserialize::deserialize(deserializer)?;
    convert_homeownership(ho_str).map_err(serde::de::Error::custom)
}

pub fn deserialize_income_verification<'de, D>(
    deserializer: D,
) -> Result<IncomeVerification, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let iv_str: &str = Deserialize::deserialize(deserializer)?;
    convert_income_verification(iv_str).map_err(serde::de::Error::custom)
}

pub fn deserialize_na_col<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let col_val: &str = Deserialize::deserialize(deserializer)?;
    handle_na_col(col_val).map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_homeownership() {
        const TEST_STR: &str = "MORTGAGE";
        let converted_ho = convert_homeownership(TEST_STR).unwrap();
        assert_eq!(converted_ho, HomeOwnership::Mortgage);
    }

    #[test]
    fn test_convert_convert_income_verification() {
        const TEST_STR: &str = "Source Verified";
        let converted_ho = convert_income_verification(TEST_STR).unwrap();
        assert_eq!(converted_ho, IncomeVerification::SourceVerified);
    }
}
