use std::sync::Arc;

use askama::Template;
use axum::{http::StatusCode, response::IntoResponse, routing::get, Router, debug_handler};

use crate::{users::AuthSession, models::auth::CurrentUser};

use super::AppState;

#[derive(Template)]
#[template(path = "protected.html")]
struct ProtectedTemplate<'a> {
    username: &'a str,
    user: Option<CurrentUser>,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(self::get::protected))
}

mod get {
    use axum::Extension;

    use super::*;

    pub async fn protected(auth_session: AuthSession) -> impl IntoResponse {
        match auth_session.user {
            Some(user) => {
                let current_user = CurrentUser {username: user.username.clone(), email: user.email};
                ProtectedTemplate {
                    username: &user.username,
                    user: Some(current_user),
                }
            }
            .into_response(),

            None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}