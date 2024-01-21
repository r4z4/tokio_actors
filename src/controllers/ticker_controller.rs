use std::collections::HashMap;

use askama::Template;
use askama_axum::IntoResponse;
use axum::{Extension, Json, response::Response, debug_handler};
use futures_util::{StreamExt, SinkExt};
use futures_util::stream::SplitStream;
use hyper::StatusCode;
use reqwest::header::HeaderMap;
use serde_json::{Value, json};
use sqlx::PgPool;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::oneshot;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{connect_async, WebSocketStream};
use tokio_tungstenite::tungstenite::Message;
use crate::{error::AppError, models::{self, offer::Offer}, actors::actor::{ActorHandle, ActorMessage}};

#[derive(Debug)]
pub struct TickerTemplate {
    // pub offers: &'a HashMap<i32, Vec<Offer>>,
    pub text: String,
}

#[debug_handler]
pub async fn get_ticker(
    // Json(application): Json<models::Application>,
    Extension(pool): Extension<PgPool>,
) -> Response {
    // if application.email.is_empty() || application.password.is_empty() {
    //     return Err(AppError::MissingCredential("test".to_owned()));
    // }

    let kraken = "wss://ws.kraken.com/";
    println!("Connecting to - {}", kraken);
    let (ws_stream, _) = connect_async(kraken).await.expect("Failed to connect");
    println!("Connected to Network");
    let (mut write, mut read) = ws_stream.split();

    let read_handle = tokio::spawn(handle_incoming_messages(read));

    let subscribe_msg = Message::Text(json!({
        "event": "subscribe",
        "pair": ["XBT/USD"],
        "subscription": json!({"name": "*"})
      }).to_string());
    println!("Sending message - {}", subscribe_msg);
    write.send(subscribe_msg).await.expect("Failed to send message");
    
    // let _ = tokio::try_join!(read_handle);
    sleep(Duration::from_millis(3000)).await;
    let _ = tokio::try_join!(read_handle);
    TickerTemplate {text: "Hi".to_owned()}.into_response()

}

async fn handle_message(mut read: SplitStream<WebSocketStream<impl AsyncRead + AsyncWrite + Unpin>>) -> impl IntoResponse {
    while let Some(message) = read.next().await {
        match message {
            Ok(msg) => TickerTemplate {text: msg.to_string()}.into_response(),
            Err(e) => TickerTemplate {text: e.to_string()}.into_response(),
        };
    }
}

async fn handle_incoming_messages(mut read: SplitStream<WebSocketStream<impl AsyncRead + AsyncWrite + Unpin>>) {
    while let Some(message) = read.next().await {
        match message {
            Ok(msg) => println!("Received a message: {}", msg),
            Err(e) => eprintln!("Error receiving message: {}", e),
        }
    }
}

impl IntoResponse for TickerTemplate {
    fn into_response(self) -> axum::response::Response {
        let mut headers = HeaderMap::new();
        headers.insert("HX-Retarget", "#ticker_data".parse().unwrap());
        (StatusCode::CREATED, self.text).into_response()
    }
}