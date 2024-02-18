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
#[template(path = "login.html")]
pub struct LoginTemplate {
    user: Option<CurrentUser>,
    message: Option<String>,
    next: Option<String>,
}

#[derive(Template)]
#[template(path = "register.html")]
pub struct RegisterTemplate {
    user: Option<CurrentUser>,
    message: Option<String>,
}

// This allows us to extract the "next" field from the query string. We use this
// to redirect after log in.
#[derive(Debug, Deserialize)]
pub struct NextUrl {
    next: Option<String>,
}

pub fn router() -> Router<Arc<Mutex<SharedState>>> {
    Router::new()
        .route("/login", post(self::post::login))
        .route("/login", get(self::get::login))
        .route("/register", post(self::post::register))
        .route("/register", get(self::get::register))
        .route("/logout", get(self::get::logout))
}

mod post {
    use std::sync::Mutex;

    use axum::{extract::State, Extension};

    use super::*;

    pub async fn login(
        mut auth_session: AuthSession,
        State(state): State<Arc<Mutex<SharedState>>>,
        Form(creds): Form<Credentials>,
    ) -> impl IntoResponse {
        let user = match auth_session.authenticate(creds.clone()).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                return LoginTemplate {
                    message: Some("Invalid credentials.".to_string()),
                    next: creds.next,
                    user: None,
                }
                .into_response()
            }
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

        if auth_session.login(&user).await.is_err() {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        if let Some(ref next) = creds.next {
            Redirect::to(next).into_response()
        } else {
            Redirect::to("/").into_response()
        }
    }

    pub async fn register(
        mut auth_session: AuthSession,
        State(state): State<Arc<Mutex<SharedState>>>,
        Form(creds): Form<Credentials>,
    ) -> impl IntoResponse {
        let user = match auth_session.authenticate(creds.clone()).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                return LoginTemplate {
                    message: Some("Invalid credentials.".to_string()),
                    next: creds.next,
                    user: None,
                }
                .into_response()
            }
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

        if auth_session.login(&user).await.is_err() {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        Redirect::to("/").into_response()
    }
}

mod get {
    use super::*;

    pub async fn login(Query(NextUrl { next }): Query<NextUrl>) -> LoginTemplate {
        LoginTemplate {
            message: None,
            next,
            user: None,
        }
    }

    pub async fn register() -> RegisterTemplate {
        RegisterTemplate {
            message: None,
            user: None,
        }
    }

    pub async fn logout(mut auth_session: AuthSession) -> impl IntoResponse {
        match auth_session.logout().await {
            Ok(_) => Redirect::to("/login").into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
