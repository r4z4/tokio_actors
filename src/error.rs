use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    GenericError(String),
    InvalidToken,
    InternalServerError,
    MissingCredential(String),
    ConfigMissingEnv(String),
    FailToCreatePool,
    TokenCreation,
    UserDoesNotExist,
    UserAlreadyExists(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GenericError(s) => write!(f, "{}", s),
            Self::InternalServerError => write!(f, "Internal server error"),
            Self::InvalidToken => write!(f, "invalid token"),
            Self::MissingCredential(s) => write!(f, "Missing credential(s)"),
            Self::ConfigMissingEnv(s) => write!(f, "Unable to find Env Var"),
            Self::FailToCreatePool => write!(f, "Failed to create DB Pool"),
            Self::TokenCreation => write!(f, "Failed to create a token"),
            Self::UserDoesNotExist => write!(f, "No user found"),
            Self::UserAlreadyExists(s) => write!(f, "User already exists"),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, err_msg) = match self {
            Self::GenericError(s) => (StatusCode::INTERNAL_SERVER_ERROR, "Generic Error"),
            Self::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
            Self::InvalidToken => (StatusCode::BAD_REQUEST, "invalid token"),
            Self::MissingCredential(s) => (StatusCode::BAD_REQUEST, "Missing credential(s)"),
            Self::ConfigMissingEnv(s) => (StatusCode::BAD_REQUEST, "Unable to find Env Var"),
            Self::FailToCreatePool => (StatusCode::BAD_REQUEST, "Failed to create DB Pool"),
            Self::TokenCreation => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create a token",
            ),
            Self::UserDoesNotExist => (StatusCode::UNAUTHORIZED, "No user found"),
            Self::UserAlreadyExists(s) => (StatusCode::BAD_REQUEST, "User already exists"),
        };
        (status, Json(json!({ "error": err_msg }))).into_response()
    }
}

impl std::error::Error for AppError {
    fn description(&self) -> &str {
        match self {
            Self::GenericError(s) => "Gerneic error",
            Self::InternalServerError => "Internal server error",
            Self::InvalidToken => "invalid token",
            Self::MissingCredential(s) => "Missing credential(s)",
            Self::ConfigMissingEnv(s) => "Unable to find Env Var",
            Self::FailToCreatePool => "Failed to create DB Pool",
            Self::TokenCreation => "Failed to create a token",
            Self::UserDoesNotExist => "No user found",
            Self::UserAlreadyExists(s) => "User already exists",
        }
    }
}
