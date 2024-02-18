use std::{borrow::Cow, collections::HashMap};

use validator::ValidationError;

use crate::config::ACCEPTED_PRIMARIES;

pub fn validate_primary_address(addr: &str) -> Result<(), ValidationError> {
    if !addr.contains(" ") {
        return Err(ValidationError {
            code: std::borrow::Cow::Borrowed("contain"),
            message: Some(Cow::from("Primary Address must contain a space.")),
            params: HashMap::new(),
        });
    }
    let street_strings: Vec<&str> = addr.split(" ").collect::<Vec<&str>>().to_owned();
    let ss_len = street_strings.len();
    // Getting last two to account for 101 Hartford St. W etc..
    if ACCEPTED_PRIMARIES.contains(&street_strings[ss_len - 1])
        || ACCEPTED_PRIMARIES.contains(&street_strings[ss_len - 2])
    {
        Ok(())
    } else {
        Err(ValidationError {
            code: std::borrow::Cow::Borrowed("identifier"),
            message: Some(Cow::from(
                "Primary Address must contain a valid Identifier (St., Ave, Lane ...)",
            )),
            params: HashMap::new(),
        })
    }
}

pub fn validate_amount(amt: i32) -> Result<(), ValidationError> {
    let offer_range = 2000..70000;
    if offer_range.contains(&amt) {
        Ok(())
    } else {
        return Err(ValidationError {
            code: std::borrow::Cow::Borrowed("amount"),
            message: Some(Cow::from("Amount must be positive")),
            params: HashMap::new(),
        });
    }
}
