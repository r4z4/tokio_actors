use askama::Template;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

use crate::config::{employment_options, marital_status_options, purpose_options, FormErrorResponse, SelectOption};

use super::auth::CurrentUser;

#[derive(Debug)]
pub struct Application<'a> {
    pub consultant_id: i32,
    pub employment_status: i32,
    pub location_id: i32,
    pub purpose_id: i32,
    pub homeownership: i32,
    pub address_one: &'a str,
    pub address_two: &'a str,
    pub city: &'a str,
    pub state: &'a str,
    pub zip: &'a str,
    pub phone: &'a str,
    pub ssn: &'a str,
    pub contact_id: i32,
    pub dob: &'a str,
}

#[derive(Debug, Template)]
#[template(path = "application.html")]
pub struct ApplicationTemplate<'a> {
    pub user: &'a Option<CurrentUser>,
    pub message: Option<String>,
    pub validation_errors: FormErrorResponse,
    pub location_options: &'a Vec<SelectOption>,
    pub purpose_options: &'a Vec<SelectOption>,
    pub marital_options: &'a Vec<SelectOption>,
    pub employment_options: &'a Vec<SelectOption>,
    pub consultant_options: &'a Vec<SelectOption>,
    pub homeownership_options: &'a Vec<SelectOption>,
    pub state_options: &'a Vec<SelectOption>,
    pub contact_options: &'a Vec<SelectOption>,
    pub entity: Option<Application<'a>>,
}

impl ApplicationTemplate<'_> {
    pub fn new(current_user: Option<CurrentUser>) -> Self {
        let s_opts = vec![SelectOption { key: "One".to_owned(), value: 1 }, SelectOption { key: "Two".to_owned(), value: 2 }];
        let app = Application {consultant_id:1,employment_status:1,location_id:1,city:"",purpose_id:1,homeownership:1,ssn:"444-33-4444", dob: "", address_one: "", address_two: "", state: "", zip: "", phone: "", contact_id: 1};
        ApplicationTemplate {
            user: &current_user,
            message:None,
            location_options: &s_opts,
            consultant_options: &s_opts,
            marital_options: &marital_status_options(), 
            employment_options: &employment_options(),
            purpose_options: &purpose_options(),
            homeownership_options: &s_opts, 
            entity: None, 
            validation_errors: FormErrorResponse { errors: None }, 
            state_options: &s_opts, 
            contact_options: &s_opts
        }
    }
}