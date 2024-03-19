use chrono::{Datelike, NaiveDate, Utc};
use csv::Reader;
use fastembed::TextEmbedding;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use sqlx::{FromRow, PgPool};
use std::collections::HashMap;
use std::fmt;
use std::{fs::File, io::Write};
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio::time::{sleep, Duration};

use crate::models::credit_file::CreditFile;
use crate::models::offer::Offer;

pub struct Actor {
    receiver: mpsc::Receiver<ActorMessage>,
    next_id: u32,
}

#[derive(Serialize, Deserialize)]
pub struct ActorResponse {
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreateActor {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoopInstructions {
    pub iterations: i32,
    // FIXME: Not string
    pub listen_for: Option<String>,
}

#[derive(Debug)]
pub enum ActorMessage {
    GetUniqueId {
        respond_to: oneshot::Sender<u32>,
    },
    GetOffers {
        respond_to: Option<oneshot::Sender<ActorMessage>>,
        offers: Option<HashMap<i32, Vec<Offer>>>,
    },
    FetchSimilars {
        respond_to: Option<oneshot::Sender<ActorMessage>>,
        embeddings: Option<Vec<Vec<f32>>>,
        similars: Option<Vec<EmbeddingSimilarsResponse>>,
        pool: Option<PgPool>,
    },
    GetOffersMpsc {
        respond_to: Option<mpsc::Sender<ActorMessage>>,
        offers: Option<HashMap<i32, Vec<Offer>>>,
    },
    GetOffersLoop {
        respond_to: Option<broadcast::Sender<String>>,
        offers: Option<HashMap<i32, Vec<Offer>>>,
        self_pid: ActorHandle,
        instructions: LoopInstructions,
    },
    RegularMessage {
        text: String,
        // respond_to: Option<mpsc::Sender<ActorMessage>>,
    },
    PopulateDB {
        text: String,
        respond_to: Option<oneshot::Sender<ActorMessage>>,
    },
}

pub fn aggregate_offers(num_lenders: i32) -> HashMap<i32, Vec<Offer>> {
    let mut offers_map = HashMap::new();
    for n in 0..num_lenders {
        dbg!(&n);
        let offers = get_mock_offers(3);
        let servicer_id = offers[0].servicer_id;
        offers_map.insert(servicer_id, offers);
    }
    offers_map
}

pub fn get_mock_offers(num_offers: i32) -> Vec<Offer> {
    // Want all same servicer
    let servicer_id = rand::thread_rng().gen_range(0..2);
    let offers = (0..num_offers)
        .map(|_| {
            let offer = mock_offer(servicer_id);
            offer
        })
        .into_iter()
        .collect::<Vec<Offer>>();
    dbg!(&offers);
    offers
}

pub fn mock_offer(servicer_id: i32) -> Offer {
    let mut rng = rand::thread_rng();
    let exp_dt = Utc::now() + chrono::Duration::days(21);
    let terms = [12, 24, 36, 48, 64, 78, 96, 128];
    let test_mins = [2000, 4000, 5000, 10000];
    let test_maxes = [20000, 35000, 55000, 75000];
    let percent_fees = [1.5, 2.5, 3.3, 4.2, 5.3];
    let aprs = [6.0, 6.8, 7.2, 8.4, 9.6, 12.4, 14.7];
    Offer {
        offer_slug: Uuid::new_v4().to_string(),
        servicer_id: servicer_id,
        max_amount: test_mins[rand::thread_rng().gen_range(0..test_mins.len())],
        min_amount: test_maxes[rand::thread_rng().gen_range(0..test_maxes.len())],
        terms: terms[rand::thread_rng().gen_range(0..terms.len())],
        percent_fee: percent_fees[rand::thread_rng().gen_range(0..percent_fees.len())],
        apr: aprs[rand::thread_rng().gen_range(0..aprs.len())],
        expires: NaiveDate::from_ymd(exp_dt.year(), exp_dt.month(), exp_dt.day()),
    }
}

#[derive(Clone, Debug)]
pub struct ActorHandle {
    pub sender: mpsc::Sender<ActorMessage>,
}

#[derive(Debug, Deserialize, FromRow)]
pub struct EmbeddingSimilarsResponse {
    pub entry_name: String,
}

impl Actor {
    async fn run(&mut self) {
        println!("Actor has spawned");
        while let Some(msg) = self.receiver.recv().await {
            Actor::handle_message(self, msg).await;
        }
    }
    fn new(receiver: mpsc::Receiver<ActorMessage>) -> Self {
        Actor {
            receiver,
            next_id: 0,
        }
    }
    async fn handle_message(&mut self, msg: ActorMessage) {
        println!("handle_message");
        match msg {
            ActorMessage::GetUniqueId { respond_to } => {
                println!("Get Unique ID has been received");
                self.next_id += 1;

                // The `let _ =` ignores any errors when sending.
                //
                // This can happen if the `select!` macro is used
                // to cancel waiting for the response.
                let _ = respond_to.send(self.next_id);
            }
            ActorMessage::GetOffers { respond_to, offers } => {
                println!("GetOffers received");
                let offers = aggregate_offers(3);
                self.next_id += 1;
                let seconds = rand::thread_rng().gen_range(2..7);
                let sec_opts = [3, 12];
                let seconds = sec_opts[rand::thread_rng().gen_range(0..sec_opts.len())];
                dbg!(seconds);
                sleep(Duration::from_millis(seconds * 1000)).await;
                let actor_message = ActorMessage::GetOffers {
                    respond_to: None,
                    offers: Some(offers),
                };
                if let Some(sender) = respond_to {
                    let _ = sender.send(actor_message);
                }
            }
            ActorMessage::GetOffersMpsc { respond_to, offers } => {
                println!("GetOffersMpsc received");
                let offers = aggregate_offers(3);
                self.next_id += 1;
                let seconds = rand::thread_rng().gen_range(2..7);
                let sec_opts = [3, 12];
                let seconds = sec_opts[rand::thread_rng().gen_range(0..sec_opts.len())];
                dbg!(seconds);
                sleep(Duration::from_millis(seconds * 1000)).await;
                let actor_message = ActorMessage::GetOffers {
                    respond_to: None,
                    offers: Some(offers),
                };
                if let Some(sender) = respond_to {
                    let _ = sender.send(actor_message);
                }
            }
            ActorMessage::GetOffersLoop {
                respond_to,
                offers,
                self_pid,
                instructions,
            } => {
                println!(
                    "GetOffersLoop received. Iterations Left: {}",
                    instructions.iterations
                );
                let count = instructions.iterations - 1;
                let str = format!("From Loop: {}", count);
                let tx = respond_to.clone().unwrap();
                let _ = tx.send(str);
                let new_instructions = LoopInstructions {
                    iterations: count,
                    listen_for: None,
                };
                let offers = aggregate_offers(3);
                sleep(Duration::from_millis(2000)).await;
                // If want to loop, create message & send
                if count > 0 {
                    let loop_message = ActorMessage::GetOffersLoop {
                        respond_to: respond_to,
                        offers: Some(offers),
                        self_pid: self_pid.clone(),
                        instructions: new_instructions,
                    };
                    self_pid.sender.send(loop_message).await;
                }
            }
            ActorMessage::RegularMessage { text } => {
                println!("Regular Message has been received: {}", text);
                sleep(Duration::from_millis(9000)).await;
                self.next_id += 1;
                println!("And after 9 seconds, next_id is: {}", self.next_id);

                // let file = File::create("foo.txt");
                // let _ = file.unwrap().write_all(b"Hello, world!");

                // The `let _ =` ignores any errors when sending.
                //
                // This can happen if the `select!` macro is used
                // to cancel waiting for the response.
                // let msg = ActorMessage::RegularMessage { text: "Hey".to_owned() };
                // let _ = respond_to.unwrap().send(msg);
            }
            ActorMessage::PopulateDB { respond_to, text } => {
                let file_name = "assets/data/____credit_file.csv";
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
                let mut rows = rdr
                    .deserialize()
                    .map(|r| r.unwrap())
                    .collect::<Vec<CreditFile>>();

                rows.iter()
                    .take(20)
                    .for_each(|r| println!("{:?} & {:?}", r.emp_title, r.months_since_last_delinq));
            }
            ActorMessage::FetchSimilars { respond_to, embeddings, similars, pool } => {
                println!("fetch huh");
                let emb = embeddings.unwrap();
                let first = &emb[0];
                // Save one to DB
                match sqlx::query_as::<_, EmbeddingSimilarsResponse>(
                    "SELECT entry_name FROM writing_sample ORDER BY embedding <-> $1 LIMIT 5;",
                )
                .bind(&first)
                .fetch_all(&pool.unwrap())
                .await
                {
                    Ok(entries) => {
                        // return (StatusCode::CREATED, ApplyOffersTemplate { message: "Hey" }).into_response()
                        dbg!(&entries);
                        let actor_message = ActorMessage::FetchSimilars {
                            respond_to: None,
                            embeddings: None,
                            pool: None,
                            similars: Some(entries),
                        };
                        if let Some(sender) = respond_to {
                            let _ = sender.send(actor_message);
                        }
                    }
                    Err(err) => {
                        dbg!(&err);
                        // let user_alert = UserAlert::from((format!("Error adding location: {:?}", err).as_str(), "alert_error"));
                        // return StatusCode::INTERNAL_SERVER_ERROR.into_response()
                    }
                }
            }
        }
    }
}

impl ActorHandle {
    pub fn new() -> Self {
        let task_monitor = tokio_metrics::TaskMonitor::new();
        let (sender, receiver) = mpsc::channel(8);
        let mut actor = Actor::new(receiver);
        tokio::task::Builder::new().name("actor_task").spawn(task_monitor.clone().instrument(async move { actor.run().await }));
        Self { sender }
    }
}
