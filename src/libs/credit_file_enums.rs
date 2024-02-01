use std::str::FromStr;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::models::credit_file::HomeOwnership;

impl Into<u32> for HomeOwnership {
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

pub fn convert_homeownership(ho_string: &str) -> Result<HomeOwnership, &'static str> {
    HomeOwnership::from_str(ho_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_homeownership() {
        const HO: &str = "MORTGAGE";
        let converted_ho = convert_homeownership(HO).unwrap();
        assert_eq!(converted_ho, HomeOwnership::Mortgage);
    }
}