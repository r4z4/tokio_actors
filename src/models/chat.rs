use std::fmt;

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, Debug, Default, Clone, FromRow)]
pub struct Room {
    pub room_id: i32,
    pub room_name: String,
    pub created_by: i32,
}

impl fmt::Display for Room {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(f, "{}:{}:{}", self.room_id, self.room_name, self.created_by)
    }
}