use redis::RedisError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApiError {
    pub error_text: String,
    pub error_class: i32,
}

impl From<(&str, i32)> for ApiError {
    fn from(pair: (&str, i32)) -> Self {
        let (text, class) = pair;
        ApiError {
            error_text: text.to_string(),
            error_class: class,
        }
    }
}

impl From<(RedisError)> for ApiError {
    fn from(e: RedisError) -> Self {
        ApiError {
            error_text: e.to_string(),
            error_class: 2,
        }
    }
}
