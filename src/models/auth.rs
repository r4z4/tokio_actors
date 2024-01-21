use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::config::SelectOption;

// #[derive(Serialize, Default, Deserialize, Debug, Clone, PartialEq)]
// // #[serde(rename_all = "camelCase")]
// enum UserType {
//     #[default]
//     Guest,
//     RegularUser,
//     Admin
// }

#[derive(Serialize, Deserialize, Debug, Default, Clone, FromRow)]
// #[serde(rename_all = "camelCase")]
pub struct User {
    pub user_id: i32,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // user_type: UserType,
}

pub struct LoginUser {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, FromRow)]
// #[serde(rename_all = "camelCase")]
pub struct UserHomeQuery {
    pub user_id: i32,
    pub user_type_id: i32,
    pub username: String,
    pub avatar_path: Option<String>,
    pub user_subs: Vec<i32>,
    pub location_subs: Vec<i32>,
    // pub first_name: Option<String>,
    // pub last_name: Option<String>,
    pub email: String,
    pub settings_updated: DateTime<Utc>,
    pub list_view: String,
    // pub theme_options: Vec<SelectOption>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // pub created_at_fmt: String,
    // pub updated_at_fmt: String,
    // user_type: UserType,
}

// Do the date format in DB layer? Or in Rust code? Let's go Rust code. Query as DateTime, then pass into template (UserModel) as String

#[derive(Serialize, FromRow, Deserialize, Debug, Default, Clone)]
// #[serde(rename_all = "camelCase")]
pub struct UserHomeModel {
    pub user_id: i32,
    pub user_type_id: i32,
    pub username: String,
    pub avatar_path: Option<String>,
    // pub first_name: Option<String>,
    // pub last_name: Option<String>,
    pub email: String,
    pub settings_updated: String,
    pub theme_options: Vec<SelectOption>,
    pub list_view_options: Vec<SelectOption>,
    pub created_at_fmt: String,
    pub updated_at_fmt: String,
    // user_type: UserType,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, FromRow)]
pub struct UserSettingsModel {
    pub user_settings_id: i32,
    pub user_id: i32,
    pub username: String,
    // user_settings table updated_at, not user table
    pub updated_at: DateTime<Utc>,
}
#[derive(Serialize, Deserialize, Debug, Default, Clone, FromRow)]
pub struct UserSettingsQuery {
    pub user_id: i32,
    pub username: String,
    pub email: String,
    pub user_updated: DateTime<Utc>,
    pub settings_updated: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, FromRow)]
pub struct UserSettingsObj {
    // user_settings table updated_at, not user table
    pub updated_at_fmt: String,
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, FromRow)]
pub struct CurrentUser {
    pub username: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, FromRow)]
pub struct CurrentUserOpt {
    pub current_user: Option<CurrentUser>,
}


#[derive(Serialize, Deserialize, Debug, Default, Clone, FromRow)]
pub struct UserSettingsPost {
    pub theme_id: i32,
    pub user_id: i32,
    pub notifications: bool,
    pub newsletter: bool,
}

/// An admin is still a user
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
struct Admin(User);