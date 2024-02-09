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
        .route("/sse", get(self::get::event_stream))
        .route("/trigger", get(self::get::trigger_call))
}

mod get {
    use std::{collections::HashMap, convert::Infallible, time::Duration};

    use axum::{extract::State, Extension};
    use chrono::NaiveDate;
    use futures_util::{stream, Stream, StreamExt};
    use rand::{distributions::Alphanumeric, Rng};
    use tokio::{spawn, sync::{broadcast, mpsc, oneshot}, time::sleep};

    use crate::{actors::actor::{get_mock_offers, mock_offer, ActorHandle, ActorMessage, LoopInstructions}, models::{credit_file::mock_credit_file, loan::mock_loan, offer::Offer}};

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
        // let mock_credit_file = mock_credit_file();
        // dbg!(mock_credit_file);
        // let mock_loan = mock_loan();
        // dbg!(mock_loan);

        // let mock_offer = mock_offer(1);
        // let _ = state.lock().unwrap().offer_tx.clone().unwrap().send(mock_offer);

        // let offer_handle = state.lock().unwrap().actor_handle.clone();
        // let (offer_event_tx, mut offer_event_rx) = broadcast::channel(5000);
        // let loop_instruction = LoopInstructions {iterations: 4, listen_for: None };
        // let offer_loop_msg = ActorMessage::GetOffersLoop {respond_to: Some(offer_event_tx), offers: None, self_pid: offer_handle.clone(), instructions: loop_instruction };
        // let _ = spawn(async move {
        //     loop {
        //         if let Ok(evt) = offer_event_rx.try_recv() {
        //             // match evt.trim() {
        //             //     // Err(Closed)
        //             //     // handle all possible actions
        //             // }
        //             println!("event: {:?}", evt);
        //         }
        //     }            
        // });
        let offer_handle = ActorHandle::new();
        let (send, recv) = oneshot::channel();
        let offer_msg = ActorMessage::PopulateDB {respond_to: Some(send), text: "".to_owned() };
        let _ = offer_handle.sender.send(offer_msg).await;
        // let _ = offer_handle.sender.send(offer_loop_msg).await;
        // state.lock().unwrap().offer_tx.send(rand::thread_rng().sample_iter(&Alphanumeric).take(5).map(char::from).collect::<String>());
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

    pub async fn event_stream(
        State(state): State<Arc<Mutex<SharedState>>>,
    ) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
        // let offer_handle = ActorHandle::new();
        // let (send, mut recv) = mpsc::channel(8);

        let (offer_tx, mut offer_rx) = broadcast::channel(5000);
        // let offer_msg = ActorMessage::GetOffersBroadcast {respond_to: Some(&tx), offers: None };
        // let _ = offer_handle.sender.send(offer_msg).await;
        // let date = NaiveDate::from_ymd_opt(2025, 6, 3).unwrap();
        // let default_offer = Offer { servicer_id: 1, max_amount: 12, min_amount: 2, terms: 22, percent_fee: 1.2, apr: 2.2, expires: date };
        // let default_offers = vec![default_offer];
        let mut lock = state.lock().unwrap();
        lock.offer_tx = Some(offer_tx.clone());

        // let offers = get_mock_offers(3);
        // let offer_tx_clone = offer_tx.clone();
        // let _ = spawn(async move {
        //     for offer in offers {
        //         sleep(Duration::from_millis(5000)).await;
        //         let _ = offer_tx.send(offer);
        //     }     
        // });        

        // let what = rx.recv().await;

        // dbg!(&what);

        let events = ["new_offer", "new_event"];
    
        Sse::new(try_stream! {
            loop {
                match offer_rx.recv().await {
                    Ok(i) => {
                        let event = Event::default()
                            .event(events[rand::thread_rng().gen_range(0..events.len())])
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

