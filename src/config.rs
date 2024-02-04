use axum::extract::{self, ConnectInfo};
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use rand::distributions::{Distribution, Uniform};
use redis::{FromRedisValue, from_redis_value, RedisResult, Value, ErrorKind};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Pool, Postgres};
use std::borrow::Cow;
use std::collections::{HashMap, BTreeMap};
use std::env;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::net::{Ipv4Addr, SocketAddr};
use std::path::Path;
use std::sync::OnceLock;
use std::thread::sleep;
use std::time::{self, Instant};
use std::{fmt::Debug, net::IpAddr};
use struct_iterable::Iterable;
use validator::{Validate, ValidationError, ValidationErrors};
use std::{collections::hash_map::DefaultHasher, hash::{Hash, Hasher}};
use axum::{
    extract::{Request, Json, Extension, Query},
    routing::post,
    http::header::HeaderMap,
    body::{Bytes, Body},
    Router,
};

pub fn config() -> &'static Config {
    static INSTANCE: OnceLock<Config> = OnceLock::new();

    INSTANCE.get_or_init(|| {
        Config::load_from_env().unwrap_or_else(|ex| {
            panic!("Error Loading Config => {ex:?}")
        })
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Config {
    pub WEB_FOLDER: String,
    pub DB_URL: String,
}

impl Config {
    fn load_from_env() -> Result<Config, AppError> {
        Ok(Config {
            // Web
            WEB_FOLDER: get_env("SERVICE_WEB_FOLDER")?,
            DB_URL: get_env("DATABASE_URL")?,
        })
    }
}

fn get_env(name: &'static str) -> Result<String, AppError> {
    env::var(name).map_err(|_| AppError::ConfigMissingEnv(name.to_owned()))
}

use crate::error::AppError;
use crate::web::AppState;

lazy_static! {
    pub static ref RE_USERNAME: Regex = Regex::new(r"^[a-zA-Z0-9]{4,}$").unwrap();
    pub static ref RE_SPECIAL_CHAR: Regex = Regex::new("^.*?[@$!%*?&].*$").unwrap();
    pub static ref RE_NO_NUMBER: Regex = Regex::new("^([^0-9]*)$").unwrap();
    // HTML Validation seems to cover most of this
    // pub static ref RE_EMAIL_DOT: Regex = Regex::new(r".+[a-z]\.[a-z]+$").unwrap();
    pub static ref RE_EMAIL: Regex = Regex::new(
        r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})"
    )
    .unwrap();
    pub static ref ACCEPTED_SECONDARIES: Vec<&'static str> = vec![
        "Apt",
        "Apt.",
        "Ste",
        "Ste.",
        "Suite",
        "Apartment",
        "#",
        "Pt.",
        "No.",
        "No",
        "Unit",
        "Ut",
        "Un.",
        "Un",
        "Ut."
    ];
    pub static ref ACCEPTED_PRIMARIES: Vec<&'static str> = vec![
        "St.", "St", "Street", "Ave.", "Av.", "Ave", "Avenue", "Parkway", "Pkwy", "Pkwy.", "Dr.",
        "Dr", "Drive", "Ln", "Lane", "Ln."
    ];
}

#[derive(Serialize, Debug)]
pub struct ValidationErrorMap {
    pub key: String,
    pub errs: Vec<ValidationError>,
}

#[derive(Serialize)]
pub struct FormErrorResponse {
    pub errors: Option<Vec<ValidationErrorMap>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApiError {
    pub error_text: String,
    pub error_class: i32,
}

impl From<(&str, i32)> for ApiError {
    fn from(pair: (&str, i32)) -> Self {
        let (text, class) = pair;
        ApiError {
            error_text: text.to_string(),
            error_class: class,
        }
    }
}

#[derive(Deserialize, Clone, Serialize, Default, Debug, Validate, Iterable)]
pub struct FilterOptions {
    pub page: Option<usize>,
    pub limit: Option<usize>,
    #[validate(length(max = 36, message = "Cannot exceed 36 characters in a table search"))]
    pub search: Option<String>,
    pub key: Option<String>,
    pub dir: Option<String>,
    pub year: Option<u32>,
    pub month: Option<u32>,
}

impl From<&extract::Query<FilterOptions>> for FilterOptions {
    fn from(web_opts: &extract::Query<FilterOptions>) -> Self {
        FilterOptions {
            search: web_opts.search.clone(),
            dir: web_opts.dir.clone(),
            key: web_opts.key.clone(),
            // Doing to keep same for table
            page: Some(web_opts.page.unwrap_or(1)),
            limit: web_opts.limit,
            year: web_opts.year,
            month: web_opts.month
        }
    }
}

#[derive(Debug, Validate, Serialize, FromRow, Clone, Deserialize)]
pub struct SelectOption {
    pub value: i32,
    pub key: String,
}

#[derive(Debug, Validate, Serialize, FromRow, Clone, Deserialize)]
pub struct SelectOptionsVec {
    pub vec: Vec<SelectOption>
}

impl FromRedisValue for SelectOptionsVec {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        let v: String = from_redis_value(v)?;
        let result: Self = match serde_json::from_str::<Self>(&v) {
          Ok(v) => v,
          Err(_err) => return Err((ErrorKind::TypeError, "Parse to JSON Failed").into())
        };
        Ok(result)
    }
}

impl From<(i32, String)> for SelectOption {
    fn from(pair: (i32, String)) -> Self {
        let (value, key) = pair;
        SelectOption {
            key: key,
            value: value,
        }
    }
}

impl FromRedisValue for SelectOption {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        let v: String = from_redis_value(v)?;
        let result: Self = match serde_json::from_str::<Self>(&v) {
          Ok(v) => v,
          Err(_err) => return Err((ErrorKind::TypeError, "Parse to JSON Failed").into())
        };
        Ok(result)
    }
}

#[derive(Debug, Validate, Serialize, FromRow, Clone, Deserialize)]
pub struct StringSelectOption {
    pub value: String,
    pub key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Todo {
    pub todo: String,
    pub date: String,
}

pub fn hash_query(query: &SimpleQuery) -> u64 {
    let mut hasher = DefaultHasher::new();
    query.hash(&mut hasher);
    let query_hash = hasher.finish();
    query_hash
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct UserAlert {
    pub msg: String,
    pub alert_class: String,
}

impl From<(&str, &str)> for UserAlert {
    fn from(pair: (&str, &str)) -> Self {
        let (msg, alert_class) = pair;
        UserAlert {
            msg: msg.to_string(),
            alert_class: alert_class.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ValidationResponse {
    pub msg: String,
    pub class: String,
}

impl From<(&str, &str)> for ValidationResponse {
    fn from(pair: (&str, &str)) -> Self {
        let (msg, class) = pair;
        ValidationResponse {
            msg: msg.to_string(),
            class: class.to_string(),
        }
    }
}

pub fn test_subs() -> UserSubscriptions {
    UserSubscriptions {
        user_subs: vec![1],
        client_subs: vec![2, 3],
        consult_subs: vec![3, 4, 5],
        location_subs: vec![4, 6, 7],
        consultant_subs: vec![3, 5, 6],
    }
}

// pub fn subs_from_user(user: &ValidatedUser) -> UserSubscriptions {
//     UserSubscriptions {
//         user_subs: user.user_subs.clone(),
//         client_subs: user.client_subs.clone(),
//         consult_subs: user.consult_subs.clone(),
//         location_subs: user.location_subs.clone(),
//         consultant_subs: user.consultant_subs.clone(),
//     }
// }

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct UserSubscriptions {
    pub user_subs: Vec<i32>,
    pub client_subs: Vec<i32>,
    pub consult_subs: Vec<i32>,
    pub location_subs: Vec<i32>,
    pub consultant_subs: Vec<i32>,
}

#[derive(Serialize, Validate, FromRow, Deserialize, Debug, Default, Clone)]
pub struct State {
    state_name: String,
}

pub async fn get_state_options(pool: &Pool<Postgres>) -> Vec<StringSelectOption> {
    match sqlx::query_as::<_, State>("SELECT state_name FROM states")
        .fetch_all(pool)
        .await
    {
        Ok(state_list) => state_list
            .iter()
            .map(|state| StringSelectOption {
                key: Some(state.state_name.to_owned()),
                value: state.state_name.to_owned(),
            })
            .collect::<Vec<StringSelectOption>>(),
        Err(err) => {
            dbg!(&err);
            vec![StringSelectOption {
                key: Some("Select One".to_string()),
                value: "Select One".to_string(),
            }]
        }
    }
}

lazy_static! {
    static ref START_TIME: Instant = Instant::now();
}

#[derive(Serialize, Validate, FromRow, Deserialize, Debug, Default, Clone)]
pub struct Category {
    category_id: i32,
    category_name: String,
}

pub fn entity_name(entity_type_id: i32) -> &'static str {
    match entity_type_id {
        1 | 2 | 3 => "user",
        4 => "consultant",
        5 => "location",
        6 => "consult",
        7 => "client",
        _ => "user",
    }
}

pub async fn category_options(pool: &Pool<Postgres>) -> Vec<SelectOption> {
    match sqlx::query_as::<_, Category>("SELECT category_id, category_name FROM article_categories")
        .fetch_all(pool)
        .await
    {
        Ok(state_list) => state_list
            .iter()
            .map(|category| SelectOption {
                key: category.category_name.to_owned(),
                value: category.category_id,
            })
            .collect::<Vec<SelectOption>>(),
        Err(err) => {
            dbg!(&err);
            vec![SelectOption::from((0, "Select One".to_string()))]
        }
    }
}

pub fn mime_type_id_from_path(path: &str) -> i32 {
    let extension = Path::new(path)
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or("none");
    match extension {
        ".png" => 1,
        ".jpeg" => 2,
        ".gif" => 3,
        ".webp" => 4,
        ".svg+xml" => 5,
        ".wav" => 6,
        ".mpeg" => 7,
        ".webm" => 8,
        ".webm" => 9,
        ".mpeg" => 10,
        ".mp4" => 11,
        ".json" => 12,
        ".pdf" => 13,
        ".csv" => 14,
        ".html" => 15,
        ".ics" => 16,
        "none" => 0,
        _ => 0,
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserPostFile {
    pub default: String,
    pub description: String,
    pub title: String,
    pub posts: Vec<UserPost>,
}
#[derive(Debug, Serialize, Deserialize, Iterable, Clone)]
pub struct UserPost {
    pub slug: String,
    pub title: String,
    pub author: i32,
    pub date: String,
    pub body: String,
}

pub struct SimpleQuery {
    pub query_str: &'static str,
    pub int_args: Option<Vec<i32>>,
    pub str_args: Option<Vec<String>>,
}

impl std::hash::Hash for SimpleQuery {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.query_str.hash(state);
        self.int_args.hash(state);
        self.str_args.hash(state);
    }
}

pub fn location_contacts() -> Vec<SelectOption> {
    vec![
        SelectOption::from((1, "Location Admin".to_string())),
        SelectOption::from((2, "Site Manager".to_string())),
    ]
}

pub fn admin_user_options() -> Vec<SelectOption> {
    vec![
        SelectOption::from((1, "User 1".to_string())),
        SelectOption::from((2, "User 2".to_string())),
    ]
}

pub fn user_type_options() -> Vec<SelectOption> {
    vec![
        SelectOption::from((1, "admin".to_string())),
        SelectOption::from((2, "subadmin".to_string())),
        SelectOption::from((3, "regular".to_string())),
        SelectOption::from((4, "guest".to_string())),
    ]
}

pub fn territory_options() -> Vec<SelectOption> {
    // let file = read_yaml();
    // dbg!(&file);
    vec![
        SelectOption::from((1, "National".to_string())),
        SelectOption::from((2, "Northeast".to_string())),
        SelectOption::from((3, "West".to_string())),
        SelectOption::from((4, "Southeast".to_string())),
        SelectOption::from((5, "Midwest".to_string())),
    ]
}

pub fn consult_result_options() -> Vec<SelectOption> {
    vec![
        SelectOption::from((
            1,
            "services rendered. next meeting scheduled".to_string(),
        )),
        SelectOption::from((
            2,
            "services rendered. no follow up requested".to_string(),
        )),
    ]
}

pub fn consult_purpose_options() -> Vec<SelectOption> {
    vec![
        SelectOption::from((1, "Informational".to_string())),
        SelectOption::from((2, "Initial Service".to_string())),
        SelectOption::from((3, "Continued Service".to_string())),
        SelectOption::from((4, "Final Service".to_string())),
        SelectOption::from((5, "Audit".to_string())),
    ]
}

pub fn specialty_options() -> Vec<SelectOption> {
    vec![
        SelectOption::from((1, "Finance".to_string())),
        SelectOption::from((2, "Insurance".to_string())),
        SelectOption::from((3, "Technology".to_string())),
        SelectOption::from((4, "Government".to_string())),
    ]
}

// pub fn mock_responsive_table_data() -> ResponsiveTableData {
//     let table_headers = ["One".to_owned(), "Two".to_owned(), "Three".to_owned()].to_vec();
//     let table_row = ResponsiveTableRow {
//         tds: ["Steve".to_owned(), "Jim".to_owned(), "Lehr".to_owned()].to_vec(),
//     };
//     let table_row_2 = ResponsiveTableRow {
//         tds: ["Steve".to_owned(), "Jim".to_owned(), "Lehr".to_owned()].to_vec(),
//     };
//     let table_row_3 = ResponsiveTableRow {
//         tds: ["Steve".to_owned(), "Jim".to_owned(), "Lehr".to_owned()].to_vec(),
//     };
//     let table_row_4 = ResponsiveTableRow {
//         tds: ["Steve".to_owned(), "Jim".to_owned(), "Lehr".to_owned()].to_vec(),
//     };
//     let table_rows = [table_row, table_row_2, table_row_3, table_row_4].to_vec();
//     let responsive_table_data = ResponsiveTableData {
//         table_headers: table_headers,
//         table_rows: table_rows,
//     };

//     return responsive_table_data;
// }

/*************************
*** Validation Helpers ***
*************************/

pub fn validate_username(username: &str) -> Result<(), ValidationError> {
    if username.len() < 3 {
        Err(ValidationError {
            // FIXME: Use key? Make code a descriptor like 'length' or 'range'
            code: std::borrow::Cow::Borrowed("length"),
            message: Some(Cow::from("Username must be 3 chars.")),
            params: HashMap::new(),
        })
    } else {
        Ok(())
    }
}

pub fn validate_email(email: &str) -> Result<(), ValidationError> {
    if !email.contains(".") {
        Err(ValidationError {
            // FIXME: Use key? Make code a descriptor like 'length' or 'range'
            code: std::borrow::Cow::Borrowed("format"),
            message: Some(Cow::from("Email must contains a '.'")),
            params: HashMap::new(),
        })
    } else {
        Ok(())
    }
}

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

pub fn validate_secondary_address(addr_two: &str) -> Result<(), ValidationError> {
    // No input comes in as blank Some(""). These get turned into NULLs in DB.
    if addr_two == "" {
        return Ok(());
    }
    let len_range = 3..15;
    if !len_range.contains(&addr_two.len()) {
        Err(ValidationError {
            code: std::borrow::Cow::Borrowed("length"),
            message: Some(Cow::from("Secondary address must be 3 to 15 characters")),
            params: HashMap::new(),
        })
    } else {
        let apt_ste: Vec<&str> = addr_two.split(" ").collect::<Vec<&str>>().to_owned();
        let first = apt_ste[0];
        dbg!(&first);
        if ACCEPTED_SECONDARIES.contains(&first) {
            Ok(())
        } else {
            Err(ValidationError {
                code: std::borrow::Cow::Borrowed("identifier"),
                message: Some(Cow::from(
                    "Secondary Address must contain a valid Identifier (Unit, Apt, # ...)",
                )),
                params: HashMap::new(),
            })
            // See if I can impl From with a message
            // Err(ValidationError::new(
            //     "Secondary Address must contain a valid Identifier (Unit, Apt, # ...)",
            // ))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, FromRow)]
pub struct TableRow {
    pub th: String,
    pub tds: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, FromRow)]
pub struct FixedTableData {
    pub table_headers: Vec<String>,
    pub table_rows: Vec<TableRow>,
}

pub fn mock_fixed_table_data() -> FixedTableData {
    let table_headers = [
        "One".to_owned(),
        "Two".to_owned(),
        "Three".to_owned(),
        "Four".to_owned(),
        "Five".to_owned(),
        "Six".to_owned(),
        "Seven".to_owned(),
        "Eight".to_owned(),
        "Nine".to_owned(),
    ]
    .to_vec();
    let th = "One".to_owned();
    let tds = [
        "Two".to_owned(),
        "Three".to_owned(),
        "Four".to_owned(),
        "Five".to_owned(),
        "Six".to_owned(),
        "Seven".to_owned(),
        "Eight".to_owned(),
        "Nine".to_owned(),
    ]
    .to_vec();
    let table_row_1 = TableRow {
        th: th.clone(),
        tds: tds.clone(),
    };
    let table_row_2 = TableRow { th: th, tds: tds };
    let table_row_3 = TableRow {
        th: "One".to_owned(),
        tds: [
            "Two".to_owned(),
            "Three".to_owned(),
            "Four".to_owned(),
            "Five".to_owned(),
            "Six".to_owned(),
            "Seven".to_owned(),
            "Eight".to_owned(),
            "Nine".to_owned(),
        ]
        .to_vec(),
    };
    let table_row_4 = TableRow {
        th: "One".to_owned(),
        tds: [
            "Two".to_owned(),
            "Three".to_owned(),
            "Four".to_owned(),
            "Five".to_owned(),
            "Six".to_owned(),
            "Seven".to_owned(),
            "Eight".to_owned(),
            "Nine".to_owned(),
        ]
        .to_vec(),
    };
    let table_row_5 = TableRow {
        th: "One".to_owned(),
        tds: [
            "Two".to_owned(),
            "Three".to_owned(),
            "Four".to_owned(),
            "Five".to_owned(),
            "Six".to_owned(),
            "Seven".to_owned(),
            "Eight".to_owned(),
            "Nine".to_owned(),
        ]
        .to_vec(),
    };
    let table_row_6 = TableRow {
        th: "One".to_owned(),
        tds: [
            "Two".to_owned(),
            "Three".to_owned(),
            "Four".to_owned(),
            "Five".to_owned(),
            "Six".to_owned(),
            "Seven".to_owned(),
            "Eight".to_owned(),
            "Nine".to_owned(),
        ]
        .to_vec(),
    };
    let table_row_7 = TableRow {
        th: "One".to_owned(),
        tds: [
            "Two".to_owned(),
            "Three".to_owned(),
            "Four".to_owned(),
            "Five".to_owned(),
            "Six".to_owned(),
            "Seven".to_owned(),
            "Eight".to_owned(),
            "Nine".to_owned(),
        ]
        .to_vec(),
    };
    let table_row_8 = TableRow {
        th: "One".to_owned(),
        tds: [
            "Two".to_owned(),
            "Three".to_owned(),
            "Four".to_owned(),
            "Five".to_owned(),
            "Six".to_owned(),
            "Seven".to_owned(),
            "Eight".to_owned(),
            "Nine".to_owned(),
        ]
        .to_vec(),
    };
    let table_row_9 = TableRow {
        th: "One".to_owned(),
        tds: [
            "Two".to_owned(),
            "Three".to_owned(),
            "Four".to_owned(),
            "Five".to_owned(),
            "Six".to_owned(),
            "Seven".to_owned(),
            "Eight".to_owned(),
            "Nine".to_owned(),
        ]
        .to_vec(),
    };
    let table_row_10 = TableRow {
        th: "One".to_owned(),
        tds: [
            "Two".to_owned(),
            "Three".to_owned(),
            "Four".to_owned(),
            "Five".to_owned(),
            "Six".to_owned(),
            "Seven".to_owned(),
            "Eight".to_owned(),
            "Nine".to_owned(),
        ]
        .to_vec(),
    };
    let table_rows = [
        table_row_1,
        table_row_2,
        table_row_3,
        table_row_4,
        table_row_5,
        table_row_6,
        table_row_7,
        table_row_8,
        table_row_9,
        table_row_10,
    ]
    .to_vec();
    let fixed_table_data = FixedTableData {
        table_headers: table_headers,
        table_rows: table_rows,
    };

    return fixed_table_data;
}

pub fn get_validation_response(is_valid: Result<(), ValidationErrors>) -> FormErrorResponse {
    println!("get_validation_response firing");
    let val_errs = is_valid
        .err()
        .unwrap()
        .field_errors()
        .iter()
        .map(|x| {
            let (key, errs) = x;
            ValidationErrorMap {
                key: key.to_string(),
                errs: errs.to_vec(),
            }
        })
        .collect::<Vec<ValidationErrorMap>>();
    dbg!(&val_errs);
    // return HttpResponse::InternalServerError().json(format!("{:?}", is_valid.err().unwrap()));
    let validation_response = FormErrorResponse {
        errors: Some(val_errs),
    };
    validation_response
}

// pub async fn validate_and_get_user(
//     cookie: &actix_web::http::header::HeaderValue,
//     state: &Data<AppState>,
// ) -> Result<Option<ValidatedUser>, crate::ValError> {
//     println!("Validating {}", format!("{:?}", cookie.clone()));
//     let session_id = if cookie.to_string().split(" ").collect::<Vec<&str>>().len() > 1 {
//         cookie.to_string().split(" ").collect::<Vec<&str>>()[1].to_string()
//     } else {
//         cookie.to_string()
//     };
//     dbg!(&session_id);
//     match sqlx::query_as::<_, ValidatedUser>(
//         "SELECT username, email, user_type_id, user_subs, client_subs, consult_subs, location_subs, consultant_subs, user_settings.list_view
//         FROM users
//         LEFT JOIN user_sessions ON user_sessions.user_id = users.id
//         LEFT JOIN user_settings ON user_settings.user_id = users.id
//         WHERE session_id = $1
//         AND expires > NOW()",
//     )
//     .bind(session_id)
//     .fetch_optional(&state.db)
//     .await
//     {
//         Ok(user_option) => Ok(user_option),
//         Err(err) => {
//             dbg!(&err);
//             Err(crate::ValError {
//                 error: format!("You must not be verified: {}", err),
//             })
//         }
//     }
// }

// pub async fn redis_validate_and_get_user(
//     cookie: &actix_web::http::header::HeaderValue,
//     r_state: &Data<RedisState>,
// ) -> Result<ValidatedUser, crate::ValError> {
//     println!("Redis Validation");
//     let mut con = r_state.r_pool.get().await.unwrap();
//     match redis::cmd("GET")
//     .arg(cookie.to_string())
//     .query_async(&mut con)
//     .await
//     {
//         Ok(user) => Ok(user),
//         Err(err) => {
//             dbg!(&err);
//             Err(crate::ValError {
//                 // FIXME: msg
//                 error: format!("Not in Redis = You must not be verified: {}", err),
//             })
//         }
//     }
// }
