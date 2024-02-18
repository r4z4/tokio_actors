use std::fmt::Debug;

use serde::Deserialize;

use serde::de::DeserializeOwned;
use sqlx::error::Error;
use sqlx::postgres::PgListener;
use sqlx::Pool;
use sqlx::Postgres;
use tracing::info;

#[derive(Deserialize, Debug)]
pub enum ActionType {
    INSERT,
    UPDATE,
    DELETE,
}

#[derive(Deserialize, Debug)]
pub struct Payload {
    pub table: String,
    pub action_type: ActionType,
    pub application_slug: String,
    pub first_name: String,
}

pub async fn start_listening<T: DeserializeOwned + Sized + Debug>(
    // pool: &Pool<Postgres>,
    mut lst: PgListener,
    channels: Vec<&str>,
    call_back: impl Fn(T),
) -> Result<(), Error> {
    // let mut listener = PgListener::connect_with(pool).await.unwrap();
    // listener.listen_all(channels).await?;
    loop {
        while let Some(notification) = lst.try_recv().await? {
            info!(
                "Getting notification with payload: {:?} from channel {:?}",
                notification.payload(),
                notification.channel()
            );

            let strr = notification.payload().to_owned();
            let payload: T = serde_json::from_str::<T>(&strr).unwrap();
            info!("des payload is {:?}", payload);

            call_back(payload);
        }
    }
}
