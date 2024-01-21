use std::collections::HashMap;

use askama::Template;
use askama_axum::IntoResponse;
use axum::{Extension, Json, response::Response, debug_handler};
use hyper::StatusCode;
use serde_json::{Value, json};
use sqlx::PgPool;
use tokio::sync::oneshot;
use tokio::time::{sleep, Duration};
use crate::{error::AppError, models::{self, offer::Offer}, actors::actor::{ActorHandle, ActorMessage}};

#[derive(Debug, Template)]
#[template(path = "offers.html")]
pub struct OffersTemplate<'a> {
    pub offers: &'a HashMap<i32, Vec<Offer>>,
    pub lc_offers: Option<Vec<Offer>>,
    pub message: Option<String>,
}

#[debug_handler]
pub async fn get_offers(
    // Json(application): Json<models::Application>,
    Extension(pool): Extension<PgPool>,
) -> Response {
    // if application.email.is_empty() || application.password.is_empty() {
    //     return Err(AppError::MissingCredential("test".to_owned()));
    // }

    let offer_handle = ActorHandle::new();
    let (send, recv) = oneshot::channel();
    let offer_msg = ActorMessage::GetOffers {respond_to: Some(send), offers: None };
    let _ = offer_handle.sender.send(offer_msg).await;
    let resp = recv.await;

    // sleep(Duration::from_millis(3000)).await;

    match resp {
        // Ok(users) => (StatusCode::CREATED, Json(users)).into_response(),
        Ok(resp) => {
            match resp {
                // FIXME: Handle None in OffersTemplate
                ActorMessage::GetOffers { respond_to, offers } => {
                    let lc_offers: Option<Vec<Offer>> = offers.clone().unwrap().get(&1).cloned();
                    OffersTemplate {offers: &offers.unwrap(), lc_offers: lc_offers, message: None}.into_response()
                },
                _ => (StatusCode::CREATED, AppError::GenericError("Actor Message Type in Incorrect".to_owned())).into_response()
            }
            
        },
        Err(_) => (StatusCode::CREATED, AppError::GenericError("No Response from Actor Handler".to_owned())).into_response()
    }

}