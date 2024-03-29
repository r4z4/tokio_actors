use std::sync::{Arc, Mutex};

use askama::Template;
use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Form, Router,
};
use serde::Deserialize;

use crate::{
    models::auth::CurrentUser,
    users::{AuthSession, Credentials},
};

use super::{AppState, SharedState};

#[derive(Template)]
#[template(path = "about.html")]
pub struct AboutPageTemplate {
    msg: String,
    user: Option<CurrentUser>,
}

#[derive(Template)]
#[template(path = "contact.html")]
pub struct ContactPageTemplate {
    msg: String,
    user: Option<CurrentUser>,
}

pub fn router<'a>() -> Router<Arc<Mutex<SharedState>>> {
    Router::new()
        .route("/about", get(self::get::about_page))
        .route("/contact", get(self::get::contact_page))
}

mod get {
    use std::convert::Infallible;

    use axum::Extension;
    use futures_util::{stream, Stream, StreamExt};
    use rand::{distributions::Alphanumeric, Rng};

    use super::*;

    pub async fn about_page() -> impl IntoResponse {
        let message = "Hey There".to_string();
        AboutPageTemplate {
            msg: message,
            user: None,
        }
        .into_response()
    }

    pub async fn contact_page() -> impl IntoResponse {
        let message = "Hey Contact Page".to_string();
        ContactPageTemplate {
            msg: message,
            user: None,
        }
        .into_response()
    }
}
