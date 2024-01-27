use std::sync::Arc;

use askama::Template;
use axum::{http::StatusCode, response::IntoResponse, routing::get, Router, debug_handler};
use axum::response::sse::{Event, Sse};
use crate::{users::AuthSession, models::auth::CurrentUser};
use axum_extra::{headers, TypedHeader};
use self::get::sse_handler;
use async_stream::try_stream;


use super::AppState;
use tokio_stream::StreamExt as _;
#[derive(Template)]
#[template(path = "protected.html")]
struct ProtectedTemplate<'a> {
    username: &'a str,
    user: Option<CurrentUser>,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(self::get::protected))
        .route("/sse", get(self::get::sse_handler))
}

mod get {
    use std::{collections::HashMap, convert::Infallible};

    use axum::{extract::State, Extension};
    use chrono::NaiveDate;
    use futures_util::{stream, Stream, StreamExt};
    use rand::{distributions::Alphanumeric, Rng};
    use tokio::sync::mpsc;

    use crate::{actors::actor::{ActorHandle, ActorMessage}, models::offer::Offer};

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

    pub async fn sse_handler(
        TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
    ) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
        println!("`{}` connected", user_agent.as_str());
    
        // A `Stream` that repeats an event every second
        //
        // You can also create streams from tokio channels using the wrappers in
        // https://docs.rs/tokio-stream
        let stream = tokio_stream::StreamExt::map(stream::repeat_with(|| Event::default().data(rand::thread_rng().sample_iter(&Alphanumeric).take(5).map(char::from).collect::<String>())), Ok)
            .throttle(std::time::Duration::from_secs(1));
    
        // Sse::new(stream).keep_alive(KeepAlive::default())
        Sse::new(stream).keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(std::time::Duration::from_secs(1))
                .text(rand::thread_rng().sample_iter(&Alphanumeric).take(5).map(char::from).collect::<String>()),
        )
    }

    pub async fn event_stream(
        State(state): State<Arc<AppState>>,
    ) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
        let offer_handle = ActorHandle::new();
        let (send, mut recv) = mpsc::channel(8);
        let offer_msg = ActorMessage::GetOffersMpsc {respond_to: Some(send), offers: None };
        let _ = offer_handle.sender.send(offer_msg).await;
        let date = NaiveDate::from_ymd_opt(2025, 6, 3).unwrap();
        let default_offer = Offer { servicer_id: 1, max_amount: 12, min_amount: 2, terms: 22, percent_fee: 1.2, apr: 2.2, expires: date };
        let default_offers = vec![default_offer];
    
        Sse::new(try_stream! {
            while let Some(msg) = recv.recv().await {
                println!("Got message");
                let msg = match msg {
                    ActorMessage::GetOffersMpsc { respond_to, offers } => offers,
                    ActorMessage::GetOffers { respond_to, offers } => offers,
                    _ => None
                };
                let lc_offers: Option<Vec<Offer>> = msg.clone().unwrap().get(&1).cloned();
                let offers = lc_offers.unwrap_or(default_offers.clone());
                let first = offers.into_iter().nth(0).unwrap();
                let event = Event::default()
                    .data::<String>(first.terms.to_string());
                yield event;
            }
        }).keep_alive(axum::response::sse::KeepAlive::default())
    }
}

