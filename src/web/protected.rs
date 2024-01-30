use std::sync::Arc;

use askama::Template;
use axum::{http::StatusCode, response::IntoResponse, routing::get, Router, debug_handler};
use axum::response::sse::{Event, Sse};
use std::sync::Mutex;
use crate::{users::AuthSession, models::auth::CurrentUser};
use axum_extra::{headers, TypedHeader};
use self::get::sse_handler;
use async_stream::try_stream;


use super::AppState;
use super::SharedState;

use tokio_stream::StreamExt as _;
#[derive(Template)]
#[template(path = "protected.html")]
struct ProtectedTemplate<'a> {
    username: &'a str,
    user: Option<CurrentUser>,
}

pub fn router() -> Router<Arc<Mutex<SharedState>>> {
    Router::new()
        .route("/", get(self::get::protected))
        .route("/sse", get(self::get::sse_handler))
        .route("/trigger", get(self::get::trigger_call))
}

mod get {
    use std::{collections::HashMap, convert::Infallible, time::Duration};

    use axum::{extract::State, Extension};
    use chrono::NaiveDate;
    use futures_util::{stream, Stream, StreamExt};
    use rand::{distributions::Alphanumeric, Rng};
    use tokio::{sync::{broadcast, mpsc}, time::sleep};

    use crate::{actors::actor::{get_mock_offers, ActorHandle, ActorMessage}, models::offer::Offer};

    use super::*;

    #[debug_handler]
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

    pub async fn trigger_call(State(state): State<Arc<Mutex<SharedState>>>) -> () {
        state.lock().unwrap().event_tx.send(rand::thread_rng().sample_iter(&Alphanumeric).take(5).map(char::from).collect::<String>());
    }

    #[debug_handler]
    pub async fn sse_handler(
        TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
    ) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
        println!("`{}` connected", user_agent.as_str());
    
        // A `Stream` that repeats an event every second
        //
        // You can also create streams from tokio channels using the wrappers in
        // https://docs.rs/tokio-stream
        let stream = tokio_stream::StreamExt::map(stream::repeat_with(|| Event::default().data(rand::thread_rng().sample_iter(&Alphanumeric).take(5).map(char::from).collect::<String>())), Ok)
            .throttle(std::time::Duration::from_secs(5));
    
        // Sse::new(stream).keep_alive(KeepAlive::default())
        Sse::new(stream).keep_alive(
            axum::response::sse::KeepAlive::new()
                // Don't believe these have any effect since setting above
                .interval(std::time::Duration::from_secs(5))
                .text(rand::thread_rng().sample_iter(&Alphanumeric).take(5).map(char::from).collect::<String>()),
        )
    }

    #[debug_handler]
    pub async fn event_handler(
        State(state): State<Arc<Mutex<SharedState>>>,
    ) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
        // let offer_handle = ActorHandle::new();
        // let (send, mut recv) = mpsc::channel(8);

        let (event_2_tx, mut event_2_rx) = broadcast::channel(5000);

        let mut lock = state.lock().unwrap();
        lock.event_tx = event_2_tx;
        
        // let offers = get_mock_offers(3);
                
        // for  offer in offers {
        //     sleep(Duration::from_millis(5000));
        //     state.lock().unwrap().event_tx.send(offer.terms.to_string());
        // }
    
        Sse::new(try_stream! {
            match event_2_rx.recv().await {
                Ok(i) => {
                    let event = Event::default()
                        .data(i);

                    yield event;
                },

                Err(e) => {
                    tracing::error!(error = ?e, "Failed to get");
                }
            }
        }).keep_alive(axum::response::sse::KeepAlive::default())
    }

    pub async fn event_stream(
        State(state): State<Arc<Mutex<SharedState>>>,
    ) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
        // let offer_handle = ActorHandle::new();
        // let (send, mut recv) = mpsc::channel(8);

        let (tx, mut rx) = broadcast::channel(5000);
        // let offer_msg = ActorMessage::GetOffersBroadcast {respond_to: Some(&tx), offers: None };
        // let _ = offer_handle.sender.send(offer_msg).await;
        // let date = NaiveDate::from_ymd_opt(2025, 6, 3).unwrap();
        // let default_offer = Offer { servicer_id: 1, max_amount: 12, min_amount: 2, terms: 22, percent_fee: 1.2, apr: 2.2, expires: date };
        // let default_offers = vec![default_offer];

        let offers = get_mock_offers(3);
                
        for  offer in offers {
            sleep(Duration::from_millis(5000)).await;
            let _ = tx.send(offer);
        }

        let what = rx.recv().await;

        dbg!(&what);
    
        Sse::new(try_stream! {
            loop {
                match rx.recv().await {
                    Ok(i) => {
                        let event = Event::default()
                            .data(i.terms.to_string());
    
                        yield event;
                    },
    
                    Err(e) => {
                        tracing::error!(error = ?e, "Failed to get");
                    }
                }
            }
        }).keep_alive(axum::response::sse::KeepAlive::default())
    }
}

