use askama::Template;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

use crate::config::{employment_options, get_state_options, homeownership_options, marital_status_options, purpose_options, FormErrorResponse, SelectOption, StringSelectOption};

use super::auth::CurrentUser;

#[derive(Debug)]
pub struct Application<'a> {
    pub consultant_id: i32,
    pub employment_status: i32,
    pub emp_length: i32,
    pub desired_loan_amount: i32,
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

impl Application<'_> {
    pub fn default() -> Self {
        Application {
            consultant_id:1, 
            desired_loan_amount: 56000, 
            employment_status:1,
            emp_length: 3,
            location_id:1,
            purpose_id:1,
            homeownership:1,
            ssn:"666-66-6666", 
            dob: "07-24-1987", 
            address_one: "7724 Pine Cir", 
            address_two: "", 
            city:"Omaha",
            state: "NE", 
            zip: "68124",
            phone: "402-392-0126", 
            contact_id: 1
        }
    }
}

#[derive(Debug, Template)]
#[template(path = "application.html")]
pub struct ApplicationTemplate<'a> {
    pub user: Option<CurrentUser>,
    pub message: Option<String>,
    pub validation_errors: FormErrorResponse,
    pub purpose_options: Vec<SelectOption>,
    pub marital_options: Vec<SelectOption>,
    pub employment_options: Vec<SelectOption>,
    pub consultant_options: Vec<SelectOption>,
    pub homeownership_options: Vec<SelectOption>,
    pub state_options: Vec<StringSelectOption>,
    pub contact_options: Vec<SelectOption>,
    pub entity: Option<Application<'a>>,
}

impl ApplicationTemplate<'_> {
    pub fn default(user: Option<CurrentUser>, state_opts: Vec<StringSelectOption>) -> Self {
        ApplicationTemplate {
            user: user,
            message:None,
            consultant_options: employment_options(),
            marital_options: marital_status_options(), 
            employment_options: employment_options(),
            purpose_options: purpose_options(),
            homeownership_options: homeownership_options(), 
            entity: None, 
            validation_errors: FormErrorResponse { errors: None }, 
            state_options: state_opts, 
            contact_options: employment_options()
        }
    }
    pub fn example(user: Option<CurrentUser>, state_opts: Vec<StringSelectOption>) -> Self {
        let app = Application::default();
        ApplicationTemplate {
            user: user,
            message:None,
            consultant_options: employment_options(),
            marital_options: marital_status_options(), 
            employment_options: employment_options(),
            purpose_options: purpose_options(),
            homeownership_options: homeownership_options(), 
            entity: Some(app), 
            validation_errors: FormErrorResponse { errors: None }, 
            state_options: state_opts, 
            contact_options: employment_options()
        }
    }
}