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
use chrono::Utc;
use fastembed::TextEmbedding;
use rand::Rng;
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
use rand::prelude::SliceRandom;
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
use sqlx::{types::time::Date, Pool, Postgres, QueryBuilder};
use sqlx::FromRow;
use sqlx::{
    postgres::{PgListener, PgPoolOptions},
    PgPool,
};
use std::{
    collections::{HashMap, HashSet},
    env,
    net::SocketAddr,
    sync::{Arc, Mutex, RwLock},
};
use lazy_static::lazy_static;
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
use casbin::prelude::*;

// mod errors;
// mod handlers;
// mod infra;
// mod routes;

#[derive(Clone)]
pub struct AppState {
    name: Option<String>,
    actor_handle: ActorHandle,
}

#[derive()]
pub struct SharedState {
    pub enforcer: Enforcer,
    pub offer_tx: Option<broadcast::Sender<Offer>>,
    pub offer_rx: Option<broadcast::Receiver<Offer>>,
    pub name: Option<String>,
    pub actor_handle: ActorHandle,
    pub user_set: Mutex<HashSet<String>>,
    // Channel used to send messages to all connected clients.
    pub tx: broadcast::Sender<String>,
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

#[derive(FromRow)]
pub struct DbMessage {
    sent_from: i32,
    sent_to: i32,
    message_text: String,
    room_id: i32,
    sent_at: chrono::DateTime<Utc>
}

#[derive(FromRow)]
pub struct DbUser {
    username: String,
    password: &'static str,
    email: String,
    first_name: String,
    last_name: String
}

lazy_static! {
    /// This is an example for using doc comment attributes
    static ref PASS: &'static str = "$argon2id$v=19$m=19456,t=2,p=1$3H44ziOiAHHPL3u5x+S+Ag$YowYSA614EokasKaa5BCx+2Dtmyf+53HE+LB3EinfiQ";
}

fn build_message() -> String {
    let first = ["Hi","Hey","What","How","Look","I","Sent","Please","Me","You","Need"].choose(&mut rand::thread_rng()).unwrap();
    let second = ["there","how","when","first","next","lets","together","smart","things","lately"].choose(&mut rand::thread_rng()).unwrap();
    let third = ["test","case","root","tiles","house","farm","then","joke","lobster","spend","fresh"].choose(&mut rand::thread_rng()).unwrap();
    let fourth = ["thanks","later","typical","see","pray","lost","dream","still","wonder","play","merge"].choose(&mut rand::thread_rng()).unwrap();

    format!("{} {} {} {}", first, second, third, fourth)
}

fn insert_messages(num: i32) -> Vec<DbMessage> {
    let mut rng = rand::thread_rng();
    // Add one to avoid 0
    let to = rng.gen_range(0..29) + 1;
    let from = rng.gen_range(0..29) + 1;
    let room = rng.gen_range(0..3) + 1;
    let sent = Utc::now() + chrono::Duration::minutes(rng.gen_range(0..60));
    let mut msgs = vec![];
    for _ in 0..num {
        msgs.push(DbMessage{sent_from: from, sent_to: to, message_text: build_message(), room_id: room, sent_at: sent})
    }
    msgs
}

fn insert_users(num: i32) -> Vec<DbUser> {
    let first = ["Jim","Steve","Sue","Jen","Rich","Ken","Larry","Lisa","Margaret","Sammy","Stan"].choose(&mut rand::thread_rng()).unwrap();
    let last = ["Rodgers","Smith","Lewis","Trumpet","Boyd","Carlson","Livers","Rivers","Ryan","Marshall","Reynolds"].choose(&mut rand::thread_rng()).unwrap();
    let mut users = vec![];
    for _ in 0..num {
        users.push(DbUser{username: format!("{}{}", first.chars().nth(3).unwrap(), last.chars().nth(3).unwrap()), password: &PASS, email: format!("{}_{}@example.com", first.chars().nth(2).unwrap().to_lowercase(), last).to_owned(), first_name: format!("{}", first), last_name: format!("{}", last)})
    }
    users
}

async fn generate_mocks(pool: &Pool<Postgres>) {
    
    let users = insert_users(40);
    let mut user_query_builder = QueryBuilder::new("INSERT INTO users(username, password, email, first_name, last_name) ");
    user_query_builder.push_values(users, |mut b, new_user| {
        b.push_bind(new_user.username).push_bind(new_user.password).push_bind(new_user.email).push_bind(new_user.first_name).push_bind(new_user.last_name);
    });
    let user_query = user_query_builder.build();
    user_query.execute(pool).await;

    let messages = insert_messages(600);
    let mut message_query_builder = QueryBuilder::new("INSERT INTO messages(sent_from, sent_to, message_text, room_id, sent_at) ");
    message_query_builder.push_values(messages, |mut b, new_msg| {
        b.push_bind(new_msg.sent_from).push_bind(new_msg.sent_to).push_bind(new_msg.message_text).push_bind(new_msg.room_id).push_bind(new_msg.sent_at);
    });
    let message_query = message_query_builder.build();
    message_query.execute(pool).await;
}


pub struct FamousEntry {
    author_id: i32,
    entry_type_id: i32,
    writing_sample: &'static str,
    embedding: Vec<f32>,
}

// (author_id, sample, category)
pub struct FamousEntryDoc(pub i32, pub &'static str, pub i32);

pub fn create_docs() -> Vec<FamousEntryDoc> {
    // 5 of each
    // FIXME: Read from file
    vec![
        FamousEntryDoc(1, "Words have no power to impress the mind without the exquisite horror of their reality.", 8),
        FamousEntryDoc(1, "I became insane, with long intervals of horrible sanity.", 10),
        FamousEntryDoc(1, "Sleep, those little slices of death — how I loathe them.", 6),
        FamousEntryDoc(1, "Years of love have been forgot, In the hatred of a minute.", 6),
        FamousEntryDoc(1, "Deep into that darkness peering, long I stood there, wondering, fearing, doubting, dreaming dreams no mortal ever dared to dream before.", 10),
        FamousEntryDoc(2, "We are all apprentices in a craft where no one ever becomes a master.", 9), 
        FamousEntryDoc(2, "The world breaks everyone, and afterward, many are strong at the broken places.", 8),
        FamousEntryDoc(2, "How little we know of what there is to know. I wish that I were going to live a long time instead of going to die today because I have learned much about life in these four days; more, I think than in all other time. I'd like to be an old man to really know. I wonder if you keep on learning or if there is only a certain amount each man can understand. I thought I knew so many things that I know nothing of. I wish there was more time.", 1), 
        FamousEntryDoc(2, "If people bring so much courage to this world the world has to kill them to break them, so of course it kills them. The world breaks every one and afterward many are strong at the broken places. But those that will not break it kills. It kills the very good and the very gentle and the very brave impartially. If you are none of these you can be sure it will kill you too but there will be no special hurry.", 3),
        FamousEntryDoc(2, "In the morning I walked down the Boulevard to the rue Soufflot for coffee and brioche. It was a fine morning. The horse-chestnut trees in the Luxembourg gardens were in bloom. There was the pleasant early-morning feeling of a hot day.", 1), 
        FamousEntryDoc(3, "Maybe that's what life is... a wink of the eye and winking stars.", 10),
        FamousEntryDoc(3, "Because in the end, you won't remember the time you spent working in the office or mowing your lawn. Climb that goddamn mountain.", 9),
        FamousEntryDoc(3, "So therefore I dedicate myself, to my art, my sleep, my dreams, my labours, my suffrances, my loneliness, my unique madness, my endless absorption and hunger because I cannot dedicate myself to any fellow being.", 8),
        FamousEntryDoc(3, "I was surprised, as always, by how easy the act of leaving was, and how good it felt. The world was suddenly rich with possibility.", 8),
        FamousEntryDoc(3, "My whole wretched life swam before my weary eyes, and I realized no matter what you do it's bound to be a waste of time in the end so you might as well go mad.", 6),
        FamousEntryDoc(4, "I don't want to repeat my innocence. I want the pleasure of losing it again.", 8),
        FamousEntryDoc(4, "I hope she'll be a fool -- that's the best thing a girl can be in this world, a beautiful little fool.", 5),
        FamousEntryDoc(4, "I fell in love with her courage, her sincerity, and her flaming self respect. And it's these things I'd believe in, even if the whole world indulged in wild suspicions that she wasn't all she should be. I love her and it is the beginning of everything.", 5),
        FamousEntryDoc(4, "And so with the sunshine and the great bursts of leaves growing on the trees, just as things grow in fast movies, I had that familiar conviction that life was beginning over again with the summer.", 3),
        FamousEntryDoc(4, "I wasn't actually in love, but I felt a sort of tender curiosity.", 5),
        FamousEntryDoc(5, "If you're in trouble, or hurt or need - go to the poor people. They're the only ones that'll help - the only ones.", 8),
        FamousEntryDoc(5, "I am impelled, not to squeak like a grateful and apologetic mouse, but to roar like a lion out of pride in my profession.", 8),
        FamousEntryDoc(5, "Where does discontent start? You are warm enough, but you shiver. You are fed, yet hunger gnaws you. You have been loved, but your yearning wanders in new fields. And to prod all these there's time, the Bastard Time.", 6),
        FamousEntryDoc(5, "Sectional football games have the glory and the despair of war, and when a Texas team takes the field against a foreign state, it is an army with banners.", 8),
        FamousEntryDoc(5, "I have never smuggled anything in my life. Why, then, do I feel an uneasy sense of guilt on approaching a customs barrier?", 9),
        FamousEntryDoc(6, "When yond same star that's westward from the pole, Had made his course t'illume that part of heaven, Where now it burns, Marcellus and myself, The bell then beating one", 2),
        FamousEntryDoc(6, "The fool doth think he is wise, but the wise man knows himself to be a fool.", 7),
        FamousEntryDoc(6, "Doubt thou the stars are fire, Doubt that the sun doth move. Doubt truth to be a liar, But never doubt I love.", 7),
        FamousEntryDoc(6, "Faith, there hath been many great men that have flattered the people who ne'er loved them.", 5),
        FamousEntryDoc(6, "To thine own self be true, and it must follow, as the night the day, thou canst not then be false to any man.", 7),
        FamousEntryDoc(7, "Her own thoughts and reflections were habitually her best companions.", 10),
        FamousEntryDoc(7, "There is nothing I would not do for those who are really my friends. I have no notion of loving people by halves, it is not my nature.", 2),
        FamousEntryDoc(7, "For what do we live, but to make sport for our neighbours and laugh at them in our turn?", 3),
        FamousEntryDoc(7, "There is a stubbornness about me that never can bear to be frightened at the will of others. My courage always rises at every attempt to intimidate me.", 8),
        FamousEntryDoc(7, "Friendship is certainly the finest balm for the pangs of disappointed love.", 5),
        FamousEntryDoc(8, "When you consider things like the stars, our affairs don't seem to matter very much, do they?", 8),
        FamousEntryDoc(8, "I can only note that the past is beautiful because one never realises an emotion at the time. It expands later, and thus we don't have complete emotions about the present, only about the past.", 1),
        FamousEntryDoc(8, "As a woman I have no country. As a woman I want no country. As a woman, my country is the whole world.", 8),
        FamousEntryDoc(8, "Women have served all these centuries as looking glasses possessing the magic and delicious power of reflecting the figure of man at twice its natural size.", 6),
        FamousEntryDoc(8, "What is the meaning of life? That was all- a simple question; one that tended to close in on one with years, the great revelation had never come. The great revelation perhaps never did come. Instead, there were little daily miracles, illuminations, matches struck unexpectedly in the dark; here was one.", 1),
        FamousEntryDoc(9, "Many a book is like a key to unknown chambers within the castle of one's own self.", 10),
        FamousEntryDoc(9, "I cannot make you understand. I cannot make anyone understand what is happening inside me. I cannot even explain it to myself.", 10),
        FamousEntryDoc(9, "I write differently from what I speak, I speak differently from what I think, I think differently from the way I ought to think, and so it all proceeds into deepest darkness.", 3),
        FamousEntryDoc(9, "You do not need to leave your room. Remain sitting at your table and listen. Do not even listen, simply wait, be quiet, still and solitary. The world will freely offer itself to you to be unmasked, it has no choice, it will roll in ecstasy at your feet.", 3),
        FamousEntryDoc(9, "By believing passionately in something that still does not exist, we create it. The nonexistent is whatever we have not sufficiently desired.", 8),
    ]
}

fn generate_embeddings(docs: &Vec<FamousEntryDoc>) -> Vec<Vec<f32>> {
    let model_res = TextEmbedding::try_new(Default::default());
    let model = model_res.unwrap();
    let strings: Vec<&str> = docs.iter().map(|doc| doc.1).collect();
    let owned = strings.to_owned();
    let embeddings_res = model.embed(owned, None);
    let embeddings = embeddings_res.unwrap();
    return embeddings
}

fn famous_entries() -> Vec<FamousEntry> {
    let docs = create_docs();
    let embeddings = generate_embeddings(&docs);
    docs.iter().enumerate().map(|(idx, doc)| FamousEntry{author_id: docs[idx].0, entry_type_id: 1, writing_sample: docs[idx].1, embedding: embeddings[idx].clone()}).collect::<Vec<FamousEntry>>()
    // vec!{
    //     FamousEntry{author_id: docs[0].0, entry_type_id: 1, writing_sample: docs[0].1, embedding: embeddings[0].clone()},
    //     ...
    // }
}

// async fn insert_entries(entries: Vec<FamousEntry>, pool: &Pool<Postgres>) {
//     let mut query_builder = QueryBuilder::new("INSERT INTO famous_entries (author_id, entry_type_id, writing_sample, embedding) ");
//     query_builder.push_values(entries, |mut b, new_entry| {
//         b.push_bind(new_entry.author_id).push_bind(new_entry.entry_type_id).push_bind(new_entry.writing_sample).push_bind(new_entry.embedding);
//     });
//     let query = query_builder.build();
//     query.execute(pool).await;
// }

impl App {
    pub async fn new(run_migration: &str) -> core::result::Result<Self, Box<dyn std::error::Error>> {
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
        dbg!(&run_migration);
        // To run, if need migrations, run: ```cargo run -- init_migration```
        if run_migration == "init_migration" {
            // Why am I not able to call this from a function?
            let entries = famous_entries();
            let mut query_builder = QueryBuilder::new("INSERT INTO famous_entries (author_id, entry_type_id, writing_sample, embedding) ");
            query_builder.push_values(entries, |mut b, new_entry| {
                b.push_bind(new_entry.author_id).push_bind(new_entry.entry_type_id).push_bind(new_entry.writing_sample).push_bind(new_entry.embedding);
            });
            let query = query_builder.build();
            query.execute(&pool).await;
        }

        generate_mocks(&pool);

        Ok(Self { pool, r_pool })
    }

    pub async fn serve(self) -> core::result::Result<(), Box<dyn std::error::Error>> {
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

        let user_set = Mutex::new(HashSet::new());
        let (tx, _rx) = broadcast::channel(100);

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

        // Not thread safe. Need RWLock
        let mut e = Enforcer::new("assets/rbac_with_domains_model.conf", "assets/rbac_with_domains_policy.csv").await?;
        e.enable_log(true);
        e.enforce(("alice", "domain1", "data1", "read"))?;

        // let state = AppState { name: None, actor_handle: actor_handle.clone() };

        let state = Arc::new(Mutex::new(SharedState {
            enforcer: e,
            name: None,
            actor_handle: actor_handle.clone(),
            offer_tx: None,
            offer_rx: None,
            tx: tx,
            user_set: user_set,
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
            user_id: user.user_id,
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
