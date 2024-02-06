use std::sync::Arc;

use askama::Template;
use axum::{http::StatusCode, response::{IntoResponse, Response}, routing::get, Router, debug_handler};
use axum::response::sse::{Event, Sse};
use std::sync::Mutex;
use crate::{users::AuthSession, models::auth::CurrentUser};
use axum_extra::{headers, TypedHeader};
use async_stream::try_stream;
use axum::routing::post;


use super::AppState;
use super::SharedState;

pub fn router() -> Router<Arc<Mutex<SharedState>>> {
    Router::new()
        .route("/application", get(self::get::get_application))
        .route("/apply", post(self::post::apply))
        .route("/offer-score", get(self::get::offer_score))
}

mod post {
    use std::{collections::HashMap, convert::Infallible, time::Duration};

    use axum::{extract::State, response::Redirect, Extension, Form};
    use chrono::NaiveDate;
    use futures_util::{stream, Stream, StreamExt};
    use rand::{distributions::Alphanumeric, Rng};
    use serde::Deserialize;
    use tokio::{spawn, sync::{broadcast, mpsc}, time::sleep};

    use crate::{actors::actor::{aggregate_offers, mock_offer}, controllers::offer_controller::OffersTemplate};

    use super::*;

    #[derive(Debug, Deserialize)]
    pub struct ApplicationInput {
        pub address_one: String,
        pub address_two: String,
        pub city: String,
        pub state: String,
        pub zip: String,
        pub phone: String,
        pub ssn: String,
        pub dob: String,
        pub marital_status: i32,
        pub desired_loan_amount: i32,
        pub purpose: i32,
        pub homeownership: i32,
        pub employment_status: i32,
        pub emp_length: i32
    }

    #[debug_handler]
    pub async fn apply(
        mut auth_session: AuthSession,
        State(state): State<Arc<Mutex<SharedState>>>,
        Form(application): Form<ApplicationInput>,
    ) -> impl IntoResponse {
        match auth_session.user {
            Some(user) => {
                let current_user = CurrentUser {username: user.username.clone(), email: user.email};
                let offers = aggregate_offers(1);
                let lc_offer = mock_offer(1);
                let lc_offers = vec![lc_offer];
                dbg!(application);
                OffersTemplate {offers: &offers, lc_offers: Some(lc_offers), message: None}.into_response()
            }

            None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }

        // if let Some(ref next) = creds.next {
        //     Redirect::to(next).into_response()
        // } else {
        //     Redirect::to("/").into_response()
        // }
    }
}

mod get {
    use std::{collections::HashMap, convert::Infallible, net::SocketAddr, time::Duration};

    use axum::{extract::{ConnectInfo, Query, State}, response::Redirect, Extension, Form};
    use chrono::NaiveDate;
    use futures_util::{stream, Stream, StreamExt};
    use rand::{distributions::Alphanumeric, Rng};
    use serde::Deserialize;
    use sqlx::PgPool;
    use tokio::{spawn, sync::{broadcast, mpsc}, time::sleep};

    use crate::{config::get_state_options, error::AppError, models::{self, application::{Application, ApplicationTemplate}}};

    use super::*;

    #[debug_handler]
    pub async fn get_application(
        State(state): State<Arc<Mutex<SharedState>>>,
        ConnectInfo(addr): ConnectInfo<SocketAddr>,
        Query(params): Query<HashMap<String,String>>,
        auth_session: AuthSession,
        Extension(pool): Extension<PgPool>,
    ) -> Response {
        // let msg = ActorMessage::RegularMessage { text: "Hey from get_users()".to_owned() };
        // let _ = state.lock().unwrap().actor_handle.sender.send(msg).await;

        let users = sqlx::query_as::<_, models::auth::User>(
            "SELECT user_id, email, username, created_at, updated_at FROM users;"
        )
        .fetch_all(&pool)
        .await
        .map_err(|err| {
            dbg!(err);
            AppError::InternalServerError
        });

        let state_options = get_state_options(&pool).await;

        let current_user = 
            match auth_session.user {
                Some(user) => Some(CurrentUser {username: user.username, email: user.email}),
                _ => None,
            };

        match users {
            // Ok(users) => (StatusCode::CREATED, Json(users)).into_response(),
            Ok(users) => ApplicationTemplate::example(current_user, state_options).into_response(),
            Err(_) => (StatusCode::CREATED, AppError::InternalServerError).into_response()
        }
    }

    #[derive(Debug, Template, Deserialize)]
    #[template(path = "offer/offer_score.html")]
    pub struct OfferScoreTemplate {
        pub score: i32,
    }

    #[debug_handler]
    pub async fn offer_score(
        mut auth_session: AuthSession,
        State(state): State<Arc<Mutex<SharedState>>>,
    ) -> impl IntoResponse {
        match auth_session.user {
            Some(user) => {
                let current_user = CurrentUser {username: user.username.clone(), email: user.email};
                let score = 100;
                OfferScoreTemplate {score: score}.into_response()
            }
            .into_response(),

            None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }

        // if let Some(ref next) = creds.next {
        //     Redirect::to(next).into_response()
        // } else {
        //     Redirect::to("/").into_response()
        // }
    }
}