use std::sync::Arc;

use crate::{
    actors::actor::mock_offer,
    models::{auth::CurrentUser, offer::Offer},
    users::AuthSession,
};
use askama::Template;
use async_stream::try_stream;
use axum::response::sse::{Event, Sse};
use axum::routing::post;
use axum::{
    debug_handler,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use axum_extra::{headers, TypedHeader};
use serde::Deserialize;
use sqlx::FromRow;
use std::sync::Mutex;
use pgvector::Vector;
use super::AppState;
use super::SharedState;

pub fn router() -> Router<Arc<Mutex<SharedState>>> {
    Router::new()
        .route("/application", get(self::get::get_application))
        .route("/writing_sample", get(self::get::get_writing_sample_form))
        .route("/apply", post(self::post::apply))
        .route("/submit_sample", post(self::post::submit_sample))
        .route("/offer-score", get(self::get::offer_score))
}

fn get_comp_offer(app: ApplicationPostResponse) -> Offer {
    // let offers = aggregate_offers(1);
    let lc_offer = mock_offer(1);
    // let lc_offers = vec![&lc_offer];
    lc_offer
}

#[derive(Debug, Deserialize, FromRow)]
pub struct ApplicationPostResponse {
    pub application_id: i32,
}

mod post {
    use std::{collections::HashMap, convert::Infallible, time::Duration};

    use axum::{extract::State, http::HeaderMap, response::Redirect, Extension, Form};
    use chrono::NaiveDate;
    use fastembed::TextEmbedding;
    use futures_util::{stream, Stream, StreamExt};
    use rand::{distributions::Alphanumeric, Rng};
    use serde::Deserialize;
    use serde_json::json;
    use sqlx::{types::Uuid, FromRow, PgPool};
    use tokio::{
        spawn,
        sync::{broadcast, mpsc, oneshot},
        time::sleep,
    };
    use validator::Validate;

    use crate::{
        actors::actor::{aggregate_offers, mock_offer, ActorHandle, ActorMessage, EmbeddingSimilarsResponse},
        config::{get_validation_response, FormErrorResponse, UserAlert},
        controllers::offer_controller::OffersTemplate,
    };

    use super::*;

    #[derive(Debug, Deserialize, Validate)]
    pub struct ApplicationInput {
        pub location_id: i32,
        pub first_name: String,
        pub last_name: String,
        pub address_one: String,
        pub address_two: String,
        pub city: String,
        pub state: String,
        pub zip: String,
        pub phone: String,
        pub ssn: String,
        pub dob: String,
        pub annual_income: i32,
        pub marital_status: i32,
        pub desired_loan_amount: i32,
        pub loan_purpose: i32,
        pub homeownership: i32,
        pub employment_status: i32,
        pub emp_length: i32,
    }

    #[derive(Debug, Deserialize, Validate)]
    pub struct WritingSampleInput {
        pub entry_type_id: i32,
        pub entry_name: String,
        pub writing_sample: String,
    }

    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    #[derive(Debug, FromRow)]
    pub struct WritingSamplePostResponse {
        pub writing_sample_id: i32,
    }

    #[derive(Hash)]
    struct SsnToNacl {
        ssn: i32,
    }
    fn hash<T>(obj: T) -> u64
    where
        T: Hash,
    {
        let mut hasher = DefaultHasher::new();
        obj.hash(&mut hasher);
        hasher.finish()
    }

    #[derive(Debug, Template)]
    #[template(path = "writing_sample_thank_you.html")]
    struct WritingSampleThankYouTemplate<'a> {
        pub message: &'a str,
        pub results: &'a Vec<EmbeddingSimilarsResponse>,
    }

    #[derive(Debug, Template)]
    #[template(path = "form/form-validation.html")]
    struct FormValidationTemplate {
        form_response: FormErrorResponse,
    }

    #[derive(Debug, Template)]
    #[template(path = "apply_offers.html")]
    struct ApplyOffersTemplate<'a> {
        pub message: &'a str,
    }

    #[debug_handler]
    pub async fn apply(
        mut auth_session: AuthSession,
        State(state): State<Arc<Mutex<SharedState>>>,
        Extension(pool): Extension<PgPool>,
        Form(application): Form<ApplicationInput>,
    ) -> impl IntoResponse {
        match auth_session.user {
            Some(user) => {
                let current_user = CurrentUser::new(&user.username, &user.email);
                // let offers = aggregate_offers(1);
                // let lc_offer = mock_offer(1);
                // let lc_offers = vec![&lc_offer];
                dbg!(&application);
                let is_valid = application.validate();
                if is_valid.is_err() {
                    let validation_response = get_validation_response(is_valid);
                    let mut headers = HeaderMap::new();
                    headers.insert("HX-Retarget", "#application_errors".parse().unwrap());
                    // let body = hb
                    //     .render("forms/form-validation", &validation_response)
                    //     .unwrap();
                    // return HttpResponse::BadRequest()
                    //     .header("HX-Retarget", "#location_errors")
                    //     .body(body);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        headers,
                        FormValidationTemplate {
                            form_response: validation_response,
                        },
                    )
                        .into_response();
                } else {
                    let ssn_str = application.ssn.replace("-", "");
                    let dob = NaiveDate::parse_from_str(&application.dob, "%Y-%m-%d").unwrap();
                    dbg!(&dob);
                    // let ssn = ssn_str.parse::<i32>().unwrap();
                    // let ssn_to_nacl = SsnToNacl { ssn: ssn };
                    // let mut hasher = DefaultHasher::new();
                    // ssn_to_nacl.hash(&mut hasher);
                    // let ssn_nacl = hasher.finish();
                    // println!("{:?}", &ssn_nacl);
                    let app_slug = Uuid::new_v4().simple().to_string();
                    match sqlx::query_as::<_, ApplicationPostResponse>(
                        "INSERT INTO applications (application_slug, location_id, first_name, last_name, address_one, address_two, city, state, zip, phone, ssn_nacl, dob, marital_status, desired_loan_amount, loan_purpose, annual_income, homeownership, employment_status, emp_length) 
                                VALUES ($1, $2, $3, $4, $5, NULLIF($6, ''), $7, $8, $9, NULLIF($10, ''), DIGEST($11, 'sha256'), NULLIF($12, '1900-01-01'), $13, $14, $15, $16, $17, $18, $19) RETURNING application_id",
                    )
                    .bind(app_slug)
                    .bind(&application.location_id)
                    .bind(&application.first_name)
                    .bind(&application.last_name)
                    .bind(&application.address_one)
                    .bind(&application.address_two)
                    .bind(&application.city)
                    .bind(&application.state)
                    .bind(&application.zip)
                    .bind(&application.phone)
                    .bind(ssn_str)
                    .bind(dob)
                    .bind(&application.marital_status)
                    .bind(&application.desired_loan_amount)
                    .bind(&application.loan_purpose)
                    .bind(&application.annual_income)
                    .bind(&application.homeownership)
                    .bind(&application.employment_status)
                    .bind(&application.emp_length)
                    .fetch_one(&pool)
                    .await
                    {
                        Ok(app) => {
                            dbg!(app.application_id);
                            // Del / Invalidate Redis Key to force a DB fetch
                            // let mut con = r_state.r_pool.get().await.unwrap();
                            // let key = format!("{}:{}", "query", "location_options");
                            // let deleted: RedisResult<bool> = con.del(&key).await;
                            // match deleted {
                            //     Ok(bool) => {
                            //         println!("Key:{} -> {}", &key, {if bool {"Found & Deleted"} else {"Not Found"}});
                            //     },
                            //     Err(err) => println!("Error: {}", err)
                            // }
                            let user_alert = UserAlert::from((format!("Location added successfully: ID #{:?}", app.application_id).as_str(), "alert_success"));
                            let template_data = json!({
                                "user_alert": user_alert,
                                "user": user,
                            });

                            // return OffersTemplate {offers: &offers, lc_offers: Some(lc_offers), message: None}.into_response()
                            let _ = tokio::task::Builder::new().name("comp_offer_task").spawn(async move {
                                sleep(Duration::from_millis(5000)).await;
                                let comp_offer = get_comp_offer(app);
                                // Find comp record in credit file CSV and use that to decision on
                                // Get Id form that, then look at load CSV with that ID and see if in good shape.
                                // If so, give offer. If not, decline.
                                state.lock().unwrap().offer_tx.clone().unwrap().send(comp_offer);
                            });
                            return (StatusCode::CREATED, ApplyOffersTemplate { message: "Hey" }).into_response()
                        }
                        Err(err) => {
                            dbg!(&err);
                            let user_alert = UserAlert::from((format!("Error adding location: {:?}", err).as_str(), "alert_error"));
                            return StatusCode::INTERNAL_SERVER_ERROR.into_response()
                        }
                    }
                }
                // OffersTemplate {offers: &offers, lc_offers: Some(lc_offers), message: None}.into_response()
            }

            None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }

        // if let Some(ref next) = creds.next {
        //     Redirect::to(next).into_response()
        // } else {
        //     Redirect::to("/").into_response()
        // }
    }

    #[debug_handler]
    pub async fn submit_sample(
        mut auth_session: AuthSession,
        State(state): State<Arc<Mutex<SharedState>>>,
        Extension(pool): Extension<PgPool>,
        Form(writing_sample): Form<WritingSampleInput>,
    ) -> impl IntoResponse {
        match auth_session.user {
            Some(user) => {
                let current_user = CurrentUser::new(&user.username, &user.email);
                let is_valid = writing_sample.validate();
                if is_valid.is_err() {
                    let validation_response = get_validation_response(is_valid);
                    let mut headers = HeaderMap::new();
                    headers.insert("HX-Retarget", "#application_errors".parse().unwrap());
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        headers,
                        FormValidationTemplate {
                            form_response: validation_response,
                        },
                    )
                        .into_response();
                } else {
                    // tokio::spawn( async move {

                    // })
                    // Return an initial finding, and send out to save to DB and find_similars
                    // Generate embeddings here
                    let model_res = TextEmbedding::try_new(Default::default());
                    let model = model_res.unwrap();

                    let documents = vec![
                        &writing_sample.writing_sample
                    ];
            
                    // Generate embeddings with the default batch size, 256
                    let embeddings_res = model.embed(documents, None);
                    let embeddings = embeddings_res.unwrap();
                    // let offer_handle = ActorHandle::new();
                    // let (send, recv) = oneshot::channel::<ActorMessage>();
                    // let offer_msg = ActorMessage::FetchSimilars {
                    //     respond_to: Some(send),
                    //     embeddings: Some(embeddings.clone()),
                    //     similars: None,
                    //     pool: Some(pool.clone()),
                    // };
                    // let _ = offer_handle.sender.send(offer_msg);
                    let first = embeddings[0].clone();
                    let pool_c = pool.clone();
                    let fetch = tokio::spawn(async move {
                        use pgvector::Vector;
                        let embedding = Vector::from(first);
                        match sqlx::query_as::<_, EmbeddingSimilarsResponse>(
                            "SELECT entry_name, entry_type_id, writing_sample FROM writing_samples ORDER BY embedding <-> $1 LIMIT 5;",
                        )
                        .bind(embedding)
                        .fetch_all(&pool_c)
                        .await
                        {
                            Ok(entries) => {
                                dbg!(&entries);
                                entries
                            }
                            Err(err) => {
                                dbg!(&err);
                                vec![]
                            }
                        }
                    });
                    let one = &embeddings[0];
                    let embedding2 = Vector::from(one.clone());
                    // FIXME: Remove sub-select and add id to CurrentUser
                    match sqlx::query_as::<_, WritingSamplePostResponse>(
                        "INSERT INTO writing_samples (user_id, entry_name, entry_type_id, writing_sample, embedding) 
                                VALUES ((SELECT user_id FROM users WHERE username = $1), $2, $3, $4, $5) RETURNING writing_sample_id",
                    )
                    .bind(current_user.username)
                    .bind(&writing_sample.entry_name)
                    .bind(&writing_sample.entry_type_id)
                    .bind(&writing_sample.writing_sample)
                    .bind(embedding2)
                    .fetch_one(&pool)
                    .await
                    {
                        Ok(writing_sample_res) => {
                            dbg!(writing_sample_res.writing_sample_id);
                            let user_alert = UserAlert::from((format!("Writing Sample added successfully: ID #{:?}", writing_sample_res.writing_sample_id).as_str(), "alert_success"));
                            let template_data = json!({
                                "user_alert": user_alert,
                                "user": user,
                            });
                            let results = fetch.await;
                            let cloned = results.unwrap().clone();
                            dbg!(&cloned);
                            return (StatusCode::CREATED, WritingSampleThankYouTemplate { message: "Hey", results: &cloned }).into_response()
                        }
                        Err(err) => {
                            dbg!(&err);
                            let user_alert = UserAlert::from((format!("Error adding location: {:?}", err).as_str(), "alert_error"));
                            return StatusCode::INTERNAL_SERVER_ERROR.into_response()
                        }
                    }
                }
                // OffersTemplate {offers: &offers, lc_offers: Some(lc_offers), message: None}.into_response()
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

    use axum::{
        extract::{ConnectInfo, Query, State},
        response::Redirect,
        Extension, Form,
    };
    use chrono::NaiveDate;
    use futures_util::{stream, Stream, StreamExt};
    use rand::{distributions::Alphanumeric, Rng};
    use serde::Deserialize;
    use sqlx::PgPool;
    use tokio::{
        spawn,
        sync::{broadcast, mpsc},
        time::sleep,
    };

    use crate::{
        config::{get_entry_type_options, get_state_options, FormErrorResponse, SelectOption},
        error::AppError,
        models::{
            self,
            application::{Application, ApplicationTemplate},
        },
    };

    use super::*;

    #[debug_handler]
    pub async fn get_application(
        State(state): State<Arc<Mutex<SharedState>>>,
        ConnectInfo(addr): ConnectInfo<SocketAddr>,
        Query(params): Query<HashMap<String, String>>,
        auth_session: AuthSession,
        Extension(pool): Extension<PgPool>,
    ) -> Response {
        // let msg = ActorMessage::RegularMessage { text: "Hey from get_users()".to_owned() };
        // let _ = state.lock().unwrap().actor_handle.sender.send(msg).await;

        let users = sqlx::query_as::<_, models::auth::User>(
            "SELECT user_id, email, username, created_at, updated_at FROM users;",
        )
        .fetch_all(&pool)
        .await
        .map_err(|err| {
            dbg!(err);
            AppError::InternalServerError
        });

        let state_options = get_state_options(&pool).await;

        let current_user = match auth_session.user {
            Some(user) => Some(CurrentUser {
                username: user.username,
                email: user.email,
            }),
            _ => None,
        };

        match users {
            // Ok(users) => (StatusCode::CREATED, Json(users)).into_response(),
            Ok(users) => ApplicationTemplate::example(current_user, state_options).into_response(),
            Err(_) => (StatusCode::CREATED, AppError::InternalServerError).into_response(),
        }
    }

    #[derive(Debug)]
    pub struct WritingSample<'a> {
        pub entry_name: &'a str,
        pub entry_type_id: i32,
        pub writing_sample: &'a str,
    }
    // FIXME: Move to own module
    #[derive(Debug, Template)]
    #[template(path = "writing_sample.html")]
    pub struct WritingSampleTemplate<'a> {
        pub user: Option<CurrentUser>,
        pub message: Option<String>,
        pub validation_errors: FormErrorResponse,
        pub entry_type_options: Vec<SelectOption>,
        pub entity: Option<WritingSample<'a>>,
    }

    #[debug_handler]
    pub async fn get_writing_sample_form(
        State(state): State<Arc<Mutex<SharedState>>>,
        ConnectInfo(addr): ConnectInfo<SocketAddr>,
        Query(params): Query<HashMap<String, String>>,
        auth_session: AuthSession,
        Extension(pool): Extension<PgPool>,
    ) -> Response {
        // let msg = ActorMessage::RegularMessage { text: "Hey from get_users()".to_owned() };
        // let _ = state.lock().unwrap().actor_handle.sender.send(msg).await;

        let users = sqlx::query_as::<_, models::auth::User>(
            "SELECT user_id, email, username, created_at, updated_at FROM users;",
        )
        .fetch_all(&pool)
        .await
        .map_err(|err| {
            dbg!(err);
            AppError::InternalServerError
        });

        let entry_type_options = get_entry_type_options(&pool).await;

        dbg!(&entry_type_options);

        let current_user = match auth_session.user {
            Some(user) => Some(CurrentUser {
                username: user.username,
                email: user.email,
            }),
            _ => None,
        };

        match users {
            // Ok(users) => (StatusCode::CREATED, Json(users)).into_response(),
            Ok(users) => WritingSampleTemplate{ message: None, entity: None, validation_errors: FormErrorResponse{errors: None}, user: current_user, entry_type_options: entry_type_options}.into_response(),
            Err(_) => (StatusCode::CREATED, AppError::InternalServerError).into_response(),
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
                let current_user = CurrentUser {
                    username: user.username.clone(),
                    email: user.email,
                };
                let score = 100;
                OfferScoreTemplate { score: score }.into_response()
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
