use std::collections::HashMap;
use std::fs;
use std::ops::Deref;

use askama::Template;
use askama_axum::IntoResponse;
use axum::{Extension, Json, response::Response, debug_handler};
use csv::Reader;
use hyper::StatusCode;
use serde_json::{Value, json};
use sqlx::PgPool;
use tokio::sync::oneshot;
use tokio::time::{sleep, Duration};
use crate::models::credit_file::CreditFile;
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
                    let file_name = "assets/data/____credit_file_test_2.csv";
                    // let file_contents = fs::read_to_string(file_name).expect("Cannot read file");
                    // let mut rdr = Reader::from_reader(file_contents.as_bytes());
                    let result = Reader::from_path(file_name);
                    if result.is_err() {
                        println!("Error w/ CSV");
                        std::process::exit(9);
                    }
                    let mut rdr = result.unwrap();
                    // let mut rows = rdr.deserialize().map(|r| r.unwrap()).collect::<Vec<Review>>();
                    // for record in rdr.records() {
                    //     println!("First field is {}", record.unwrap().get(0).unwrap())
                    // }
                    let mut rows = rdr.deserialize().map(|r| r.unwrap()).collect::<Vec<CreditFile>>();
                    rows.iter().take(3).for_each(|r| println!("{:?} & {:?}", r.emp_title, r.months_since_last_delinq));
                    OffersTemplate {offers: &offers.unwrap(), lc_offers: lc_offers, message: None}.into_response()
                },
                _ => (StatusCode::CREATED, AppError::GenericError("Actor Message Type in Incorrect".to_owned())).into_response()
            }
            
        },
        Err(_) => (StatusCode::CREATED, AppError::GenericError("No Response from Actor Handler".to_owned())).into_response()
    }

}