use std::sync::Arc;

use askama::Template;
use axum::{http::StatusCode, response::IntoResponse, routing::get, Router, debug_handler};
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

    use crate::{actors::actor::{aggregate_offers, get_mock_offers, mock_offer, ActorHandle, ActorMessage, LoopInstructions}, controllers::offer_controller::OffersTemplate, models::{credit_file::mock_credit_file, loan::mock_loan, offer::Offer}, web::app::Application};

    use super::*;

    #[derive(Debug, Deserialize)]
    pub struct ApplicationInput {
        pub address_one: String,
        pub address_two: String,
        pub city: String,
        pub state: String,
        pub zip: String,
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

mod get {
    use std::{collections::HashMap, convert::Infallible, time::Duration};

    use axum::{extract::State, response::Redirect, Extension, Form};
    use chrono::NaiveDate;
    use futures_util::{stream, Stream, StreamExt};
    use rand::{distributions::Alphanumeric, Rng};
    use serde::Deserialize;
    use tokio::{spawn, sync::{broadcast, mpsc}, time::sleep};

    use crate::{actors::actor::{aggregate_offers, get_mock_offers, mock_offer, ActorHandle, ActorMessage, LoopInstructions}, controllers::offer_controller::OffersTemplate, models::{credit_file::mock_credit_file, loan::mock_loan, offer::Offer}, web::app::Application};

    use super::*;

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