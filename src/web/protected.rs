use std::sync::Arc;

use self::get::sse_handler;
use crate::{models::auth::CurrentUser, users::AuthSession};
use askama::Template;
use async_stream::try_stream;
use axum::response::sse::{Event, Sse};
use axum::{debug_handler, http::StatusCode, response::IntoResponse, routing::get, Router};
use axum_extra::{headers, TypedHeader};
use serde::Deserialize;
use sqlx::FromRow;
use tokio_metrics::RuntimeMetrics;
use std::sync::Mutex;

use super::AppState;
use super::SharedState;

use tokio_stream::StreamExt as _;
#[derive(Template)]
#[template(path = "protected.html")]
struct ProtectedTemplate<'a> {
    username: &'a str,
    user: Option<CurrentUser>,
}

#[derive(Debug, Template)]
#[template(path = "dump.html")]
pub struct DumpTemplate<'a> {
    pub dump: &'a str,
    pub metrics: &'a RuntimeMetrics,
}

#[derive(Debug, Deserialize, FromRow)]
pub struct EmbeddingPostResponse {
    pub id: i32,
}

pub fn router() -> Router<Arc<Mutex<SharedState>>> {
    Router::new()
        .route("/", get(self::get::protected))
        .route("/sse", get(self::get::event_stream))
        .route("/trigger", get(self::get::trigger_call))
        .route("/metrics", get(self::get::metrics))
}

mod get {
    use std::{collections::HashMap, convert::Infallible, time::Duration};

    use axum::{extract::State, Extension};
    use chrono::NaiveDate;
    use futures_util::{stream, Stream, StreamExt};
    use rand::{distributions::Alphanumeric, Rng};
    use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
    use sqlx::PgPool;
    use tokio::{
        spawn,
        sync::{broadcast, mpsc, oneshot},
        time::sleep,
    };

    use crate::{
        actors::actor::{get_mock_offers, mock_offer, ActorHandle, ActorMessage, LoopInstructions}, controllers::metrics_controller::task_dump, models::{credit_file::mock_credit_file, loan::mock_loan, offer::Offer}
    };

    use super::*;

    #[debug_handler]
    pub async fn protected(auth_session: AuthSession) -> impl IntoResponse {
        match auth_session.user {
            Some(user) => {
                let current_user = CurrentUser {
                    username: user.username.clone(),
                    email: user.email,
                };
                ProtectedTemplate {
                    username: &user.username,
                    user: Some(current_user),
                }
            }
            .into_response(),

            None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }

    pub async fn metrics(State(state): State<Arc<Mutex<SharedState>>>) -> impl IntoResponse {
        let handle = tokio::runtime::Handle::current();
        let runtime_monitor = tokio_metrics::RuntimeMonitor::new(&handle);
        let mut intervals = runtime_monitor.intervals();
        let runtime_metrics = intervals.next().unwrap();

        let dump_res = task_dump().await;

        match dump_res {
            Ok(dump) => (StatusCode::CREATED, DumpTemplate { dump: &dump, metrics: &runtime_metrics }).into_response(),
            Err(_) => (StatusCode::CREATED, DumpTemplate { dump: "Unable to get dump", metrics: &runtime_metrics }).into_response()
        }
    }

    pub async fn trigger_call(State(state): State<Arc<Mutex<SharedState>>>, Extension(pool): Extension<PgPool>) -> () {
        let offer_handle = ActorHandle::new();
        let (send, recv) = oneshot::channel();
        let offer_msg = ActorMessage::PopulateDB {
            respond_to: Some(send),
            text: "".to_owned(),
        };
        // With default InitOptions
        let model_res = TextEmbedding::try_new(Default::default());

        // With custom InitOptions
        // let model_res = TextEmbedding::try_new(InitOptions {
        //     model_name: EmbeddingModel::AllMiniLML6V2,
        //     show_download_progress: true,
        //     ..Default::default()
        // });

        let model = model_res.unwrap();

        let documents = vec![
            "passage: Hello, World!",
            "query: Hello, World!",
            "passage: This is an example passage.",
            // You can leave out the prefix but it's recommended
            "fastembed-rs is licensed under Apache  2.0"
        ];

        // Generate embeddings with the default batch size, 256
        let embeddings_res = model.embed(documents, None);

        let embeddings = embeddings_res.unwrap();

        println!("Embeddings length: {}", embeddings.len()); // -> Embeddings length: 4
        println!("Embedding dimension: {}", embeddings[0].len()); 

        // Save one to DB
        match sqlx::query_as::<_, EmbeddingPostResponse>(
            "INSERT INTO items (embedding) 
                    VALUES ($1) RETURNING id",
        )
        .bind(&embeddings[0])
        .fetch_one(&pool)
        .await
        {
            Ok(app) => {
                // return (StatusCode::CREATED, ApplyOffersTemplate { message: "Hey" }).into_response()
                dbg!(app);
            }
            Err(err) => {
                dbg!(&err);
                // let user_alert = UserAlert::from((format!("Error adding location: {:?}", err).as_str(), "alert_error"));
                // return StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
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
        let stream = tokio_stream::StreamExt::map(
            stream::repeat_with(|| {
                Event::default().data(
                    rand::thread_rng()
                        .sample_iter(&Alphanumeric)
                        .take(5)
                        .map(char::from)
                        .collect::<String>(),
                )
            }),
            Ok,
        )
        .throttle(std::time::Duration::from_secs(5));

        // Sse::new(stream).keep_alive(KeepAlive::default())
        Sse::new(stream).keep_alive(
            axum::response::sse::KeepAlive::new()
                // Don't believe these have any effect since setting above
                .interval(std::time::Duration::from_secs(5))
                .text(
                    rand::thread_rng()
                        .sample_iter(&Alphanumeric)
                        .take(5)
                        .map(char::from)
                        .collect::<String>(),
                ),
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

        // FIXME Comment this out to get crazy amount of tracing errors 
        let offers = get_mock_offers(3);
        let offer_tx_clone = offer_tx.clone();

        let _ = tokio::task::Builder::new().name("offer_task").spawn(async move {
            for offer in offers {
                sleep(Duration::from_millis(5000)).await;
                let _ = offer_tx.send(offer);
            }
        });

        // let what = rx.recv().await;

        // dbg!(&what);

        let events = ["new_offer", "new_event"];

        Sse::new(try_stream! {
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
        })
        .keep_alive(axum::response::sse::KeepAlive::default())
    }
}
