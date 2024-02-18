use crate::{
    actors::actor::{
        self, Actor, ActorHandle, ActorMessage, ActorResponse, CreateActor, LoopInstructions,
    },
    config::{
        employment_options, get_state_options, marital_status_options, purpose_options,
        FormErrorResponse, SelectOption,
    },
    controllers::{offer_controller::get_offers, ticker_controller::get_ticker},
    error::AppError,
    libs::pg_notify_handle::{start_listening, ActionType, Payload},
    models::{
        self,
        application::ApplicationTemplate,
        auth::{CurrentUser, CurrentUserOpt},
        offer::Offer,
        payment::CreditCardApiResp,
        store::new_db_pool,
    },
    redis_mod::redis_mod::{redis_client, redis_connect},
    users::{AuthSession, Backend},
    web::{api, auth, protected, public, ws::read_and_send_messages},
};
use ::time::Duration;
use askama::Template;
use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use axum::{
    body::Body,
    debug_handler,
    extract::{ConnectInfo, Query, State},
    http::{Request, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router,
};
use axum_login::{
    login_required,
    tower_sessions::{ExpiredDeletion, Expiry, PostgresStore, SessionManagerLayer},
    AuthManagerLayerBuilder,
};
use deadpool_redis::{redis::cmd, Pool as RedisPool};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt as _, StreamExt as _,
};
use models::auth::User;
use sendgrid::error::SendgridError;
use sendgrid::v3::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::types::time::Date;
use sqlx::FromRow;
use sqlx::{
    postgres::{PgListener, PgPoolOptions},
    PgPool,
};
use std::{
    collections::HashMap,
    env,
    net::SocketAddr,
    sync::{Arc, Mutex, RwLock},
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::{broadcast, mpsc, oneshot},
};
use tokio_cron_scheduler::{Job, JobScheduler};
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::trace::{self, TraceLayer};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use tracing::{info, Level};

// mod errors;
// mod handlers;
// mod infra;
// mod routes;

#[derive(Clone)]
pub struct AppState {
    name: Option<String>,
    actor_handle: ActorHandle,
}

#[derive(Debug)]
pub struct SharedState {
    pub offer_tx: Option<broadcast::Sender<Offer>>,
    pub offer_rx: Option<broadcast::Receiver<Offer>>,
    pub name: Option<String>,
    pub actor_handle: ActorHandle,
}

pub struct App {
    pool: PgPool,
    r_pool: RedisPool,
}

#[derive(Debug, Template)]
#[template(path = "users.html")]
pub struct UsersTemplate<'a> {
    pub users: &'a Vec<User>,
    pub message: Option<String>,
    pub user: Option<CurrentUser>,
}

#[derive(Debug, Template)]
#[template(path = "post.html")]
pub struct PostTemplate<'a> {
    pub post_title: &'a str,
    pub post_date: String,
    pub post_body: &'a str,
    pub user: Option<CurrentUser>,
}

// pub async fn pg_listen_for_app_updates(pool: &PgPool) {
//     let topic: &str = "new_app_notification";
//     match sqlx::query::<_>(
//         "LISTEN $1",
//     )
//     .bind(&topic)
//     .execute(pool)
//     .await
//     {
//         Ok(_) => println!("yay"),
//         Err(_) => println!("yay"),
//     }
// }

impl App {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv::dotenv().ok();

        // let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
        // let pool = PgPoolOptions::new()
        //     .max_connections(5)
        //     // use your own credentials
        //     .connect(&db_url)
        //     .await
        //     .expect("Unable to connect to DB");
        // sqlx::migrate!().run(&pool).await?;
        let pool = new_db_pool().await?;
        let r_pool = redis_connect();
        let current_user = None::<CurrentUser>;

        Ok(Self { pool, r_pool })
    }

    pub async fn serve(self) -> Result<(), Box<dyn std::error::Error>> {
        // println!("Serve");
        // let cors = CorsLayer::new().allow_origin(Any);

        // Get them right away here FIXME: Redis cache
        // populate_cache()

        let metrics: tokio::runtime::RuntimeMetrics = tokio::runtime::Handle::current().metrics();
        for _ in 0..10 {
            tokio::spawn(tokio::time::sleep(core::time::Duration::from_millis(10)));
        }
        let n = metrics.active_tasks_count();
        println!("Runtime has {} active tasks", n);

        let cors = CorsLayer::new()
            .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
            .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
            .allow_credentials(true)
            .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

        // let mut sched = JobScheduler::new().await?;
        // Add async job
        // sched.add(
        //     Job::new_async("1/7 * * * * *", |uuid, mut l| {
        //         Box::pin(async move {
        //             println!("I run async every 7 seconds");

        //             // Query the next execution time for this job
        //             let next_tick = l.next_tick_for_job(uuid).await;
        //             match next_tick {
        //                 Ok(Some(ts)) => println!("Next time for 7s job is {:?}", ts),
        //                 _ => println!("Could not get next tick for 7s job"),
        //             }
        //         })
        //     })?
        // ).await?;

        // sched
        //     .add(Job::new_async("0 0 1 * * 1-5", |uuid, mut l| {
        //         Box::pin(async move {
        //             println!("I run at 1 AM UTC each day. 7 PM Cen. Only on Weekdays (1-5)");

        //             // Query the next execution time for this job
        //             let next_tick = l.next_tick_for_job(uuid).await;
        //             match next_tick {
        //                 Ok(Some(ts)) => println!("Next time for 7s job is {:?}", ts),
        //                 _ => println!("Could not get next tick for 7s job"),
        //             }
        //         })
        //     })?)
        //     .await?;

        // // Add code to be run during/after shutdown
        // sched.set_shutdown_handler(Box::new(|| {
        //     Box::pin(async move {
        //         println!("Shut down done");
        //     })
        // }));

        // // Start the scheduler
        // sched.start().await?;

        // let random_data_base = "https://random-data-api.com/api/v2/";
        // let entity = "credit_cards";

        // let url = random_data_base.to_owned() + entity;

        // let resp = reqwest::get(url)
        //     .await?
        //     .json::<CreditCardApiResp>()
        //     .await?;

        // dbg!(resp);

        // let (offer_tx, mut offer_rx) = broadcast::channel(5000);

        // let client = redis_client().unwrap();
        // let mut con = client.get_connection()?;
        // let mut pubsub = con.as_pubsub();

        // pubsub.subscribe("new_trivia_question")?;

        // let stream = pubsub
        //     .get_message()
        //     .map(|m| {
        //         m.get_payload::<String>()
        //             .map_err(|e| e.to_string())
        //     });
        // .boxed();

        // let _ = redis_test_data(&r_pool).await;

        // Using GlitchTip. Works with the Rust Sentry SDK
        // let _guard = sentry::init("https://ec778decf4e94595b5a48520185298c3@app.glitchtip.com/5073");

        // tokio::spawn(async move {
        //     let resp = reqwest_client.request(req).await;
        // });

        // let url = "wss://echo.websocket.events";
        // let kraken = "wss://ws.kraken.com/";

        // let _ = pg_listen_for_app_updates(&self.pool);
        // FIXME WTF is going on here
        // let mut listener = PgListener::connect_with(&self.pool).await.unwrap();
        // listener.listen("new_app_notification");
        // tokio::spawn(async move {
        //     loop {
        //         while let Some(notification) = listener.try_recv().await? {
        //             info!(
        //                 "Getting notification with payload: {:?} from channel {:?}",
        //                 notification.payload(),
        //                 notification.channel()
        //             );

        //             // let strr = notification.payload().to_owned();
        //             // let payload: T = serde_json::from_str::<T>(&strr).unwrap();
        //             // info!("des payload is {:?}", payload);

        //             // call_back(payload);
        //         }
        //     }
        // });

        let channels = vec!["table_update", "new_app_notification"];
        let hm: HashMap<String, String> = HashMap::new();
        let constants = Arc::new(RwLock::new(hm));

        let call_back = move |payload: Payload| {
            match payload.action_type {
                ActionType::INSERT => {
                    let mut constants = constants.write().unwrap();
                    constants.insert(payload.application_slug, payload.first_name);
                }
                ActionType::UPDATE => {
                    let mut constants = constants.write().unwrap();
                    constants.insert(payload.application_slug, payload.first_name);
                }
                ActionType::DELETE => {
                    let mut constants = constants.write().unwrap();
                    constants.remove(&payload.application_slug);
                }
            };
            println!("constants: {:?}", constants);
            println!(" ");
        };
        let mut listener = PgListener::connect_with(&self.pool).await.unwrap();
        listener.listen_all(channels.clone()).await?;
        tokio::task::Builder::new().name("pg_notify_task").spawn({ start_listening(listener, channels, call_back) });

        // println!("Connecting to - {}", kraken);
        // let (ws_stream, _) = connect_async(kraken).await.expect("Failed to connect");
        // println!("Connected to Network");
        // let (mut write, mut read) = ws_stream.split();
        // // let msg = Message::Text("aloha echo server".into());
        // let read_handle = tokio::spawn(handle_incoming_messages(read));
        // // To connect to stdin
        // // let write_handle = tokio::spawn(read_and_send_messages(write));
        // // let _ = subscribe_to(&mut write);
        // let subscribe_msg = Message::Text(json!({
        //     "event": "subscribe",
        //     "pair": ["XBT/USD"],
        //     "subscription": json!({"name": "*"})
        //   }).to_string());
        // println!("Sending message - {}", subscribe_msg);
        // write.send(subscribe_msg).await.expect("Failed to send message");
        // let _ = tokio::try_join!(read_handle);
        let actor_handle = ActorHandle::new();
        let msg = ActorMessage::RegularMessage {
            text: "Hey from Main".to_owned(),
        };
        let _ = actor_handle.sender.send(msg).await;

        // let state = AppState { name: None, actor_handle: actor_handle.clone() };

        let state = Arc::new(Mutex::new(SharedState {
            name: None,
            actor_handle: actor_handle.clone(),
            offer_tx: None,
            offer_rx: None,
        }));

        let offer_handle = ActorHandle::new();
        let (send, recv) = oneshot::channel();
        let offer_msg = ActorMessage::GetOffers {
            respond_to: Some(send),
            offers: None,
        };
        // let (offer_event_tx, mut offer_event_rx) = broadcast::channel(5000);
        // let loop_instruction = LoopInstructions {iterations: 4, listen_for: None };
        // let offer_loop_msg = ActorMessage::GetOffersLoop {respond_to: Some(offer_event_tx), offers: None, self_pid: offer_handle.clone(), instructions: loop_instruction };
        // let _ = offer_handle.sender.send(offer_loop_msg).await;
        // This blocks
        // let resp = recv.await.expect("Actor task has been killed");
        // dbg!(resp);
        // let _ = tokio::try_join!(read_handle, write_handle);

        // let mut cool_header = HashMap::with_capacity(2);
        // cool_header.insert(String::from("x-cool"), String::from("indeed"));
        // cool_header.insert(String::from("x-cooler"), String::from("cold"));

        // // Personalization = Destination addr
        // let p = Personalization::new(Email::new("ar3rz3@gmail.com")).add_headers(cool_header);

        // // Create a new message from SendGrid Identity (From addr)
        // let m = sendgrid::v3::Message::new(Email::new("r4z4aa@gmail.com"))
        //     .set_subject("Subject")
        //     .add_content(
        //         Content::new()
        //             .set_content_type("text/html")
        //             .set_value("Test from Rust"),
        //     )
        //     .add_personalization(p);

        // let mut env_vars = ::std::env::vars();
        // let api_key = env_vars.find(|v| v.0 == "SENDGRID_API_KEY").unwrap();
        // let sender = Sender::new(api_key.1);
        // let resp = sender.send(&m).await?;
        // println!("status: {}", resp.status());

        // Allow bursts w/ up to 5 reqs per IP address & replenishes one element every two seconds
        // Box it b/c Axum 0.6 req all Layers to be Clone & thus we need a static reference to it
        let governor_conf = Box::new(
            GovernorConfigBuilder::default()
                .per_second(2)
                .burst_size(5)
                .finish()
                .unwrap(),
        );
        let governor_limiter = governor_conf.limiter().clone();
        let interval = std::time::Duration::from_secs(60);
        // a separate background task to clean up
        std::thread::spawn(move || loop {
            std::thread::sleep(interval);
            tracing::info!("rate limiting storage size: {}", governor_limiter.len());
            governor_limiter.retain_recent();
        });

        let session_store = PostgresStore::new(self.pool.clone());
        session_store.migrate().await?;

        let session_layer = SessionManagerLayer::new(session_store)
            .with_secure(false)
            .with_expiry(Expiry::OnInactivity(Duration::days(1)));

        // Auth service.
        //
        // This combines the session layer with our backend to establish the auth
        // service which will provide the auth session as a request extension.
        let backend = Backend::new(self.pool.clone());
        let auth_layer = AuthManagerLayerBuilder::new(backend, session_layer).build();

        let protected_app = protected::router()
            //.route_layer(login_required!(Backend, login_url = "/login"))
            .merge(api::router())
            .route("/actor", get(get_actor))
            .route("/users", get(get_users))
            .route("/offers", get(get_offers))
            .route("/ticker", get(get_ticker))
            .route_layer(login_required!(Backend, login_url = "/login"))
            // `POST /users` goes to `create_user`
            .route("/users", post(create_user))
            .merge(auth::router())
            .layer(auth_layer);
        // build our application with a route
        let app = Router::new()
            // .route("/", get(root))
            .merge(protected_app)
            .merge(public::router())
            .layer(GovernorLayer {
                // We can leak this because it is created once and then
                config: Box::leak(governor_conf),
            })
            .layer(cors)
            .layer(Extension(self.pool))
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                    .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
            )
            .with_state(state.into())
            .nest_service("/assets", ServeDir::new("assets"));
        // Routes with a different state

        // run our app with hyper, listening globally on port 3000
        let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
        tracing::debug!("Listening on {}", addr);
        // let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();

        Ok(())
    }
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

async fn subscribe_to(
    write: &mut SplitSink<WebSocketStream<impl AsyncRead + AsyncWrite + Unpin>, Message>,
) {
    println!("Firing Subscribe");
    let subscribe_msg = Message::Text(
        json!({
          "event": "subscribe",
          "pair": ["XBT/USD"],
          "subscription": json!({"name": "*"})
        })
        .to_string(),
    );
    println!("Sending message - {}", subscribe_msg);
    write
        .send(subscribe_msg)
        .await
        .expect("Failed to send message");
}

async fn handle_incoming_messages(
    mut read: SplitStream<WebSocketStream<impl AsyncRead + AsyncWrite + Unpin>>,
) {
    while let Some(message) = read.next().await {
        match message {
            Ok(msg) => println!("Received a message: {}", msg),
            Err(e) => eprintln!("Error receiving message: {}", e),
        }
    }
}

#[debug_handler]
async fn get_users(
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

    let current_user = match auth_session.user {
        Some(user) => Some(CurrentUser {
            username: user.username,
            email: user.email,
        }),
        _ => None,
    };

    match users {
        // Ok(users) => (StatusCode::CREATED, Json(users)).into_response(),
        Ok(users) => UsersTemplate {
            users: &users,
            message: None,
            user: current_user,
        }
        .into_response(),
        Err(_) => (StatusCode::CREATED, AppError::InternalServerError).into_response(),
    }
}

#[derive(Deserialize, Serialize)]
pub struct ReturnUserObject {
    pub username: String,
    pub email: Option<String>,
}

#[debug_handler]
async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    auth_session: AuthSession,
    State(state): State<Arc<Mutex<SharedState>>>,
    Query(params): Query<HashMap<String, String>>,
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<ReturnUserObject>) {
    // insert your application logic here
    let user = ReturnUserObject {
        email: None,
        username: payload.username,
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(user))
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

#[debug_handler]
async fn get_actor(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    auth_session: AuthSession,
    Extension(current_user): Extension<CurrentUserOpt>,
    Json(payload): Json<CreateActor>,
) -> (StatusCode, Json<ActorResponse>) {
    // insert your application logic here
    let resp = ActorResponse { name: payload.name };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(resp))
}
