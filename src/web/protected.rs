use std::sync::Arc;

use askama::Template;
use axum::{http::StatusCode, response::IntoResponse, routing::get, Router, debug_handler};
use axum::response::sse::{Event, Sse};
use crate::{users::AuthSession, models::auth::CurrentUser};
use axum_extra::{headers, TypedHeader};
use self::get::sse_handler;

use super::AppState;
use tokio_stream::StreamExt as _;
#[derive(Template)]
#[template(path = "protected.html")]
struct ProtectedTemplate<'a> {
    username: &'a str,
    user: Option<CurrentUser>,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(self::get::protected))
        .route("/sse", get(sse_handler))
}

mod get {
    use std::convert::Infallible;

    use axum::Extension;
    use futures_util::{stream, Stream, StreamExt};
    use rand::{distributions::Alphanumeric, Rng};

    use super::*;

    pub async fn protected(auth_session: AuthSession) -> impl IntoResponse {
        match auth_session.user {
            Some(user) => {
                let current_user = CurrentUser {username: user.username.clone(), email: user.email};
                ProtectedTemplate {
                    username: &user.username,
                    user: Some(current_user),
                }
            }
            .into_response(),

            None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }

    pub async fn sse_handler(
        TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
    ) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
        println!("`{}` connected", user_agent.as_str());
    
        // A `Stream` that repeats an event every second
        //
        // You can also create streams from tokio channels using the wrappers in
        // https://docs.rs/tokio-stream
        let stream = tokio_stream::StreamExt::map(stream::repeat_with(|| Event::default().data("hi!")), Ok)
            .throttle(std::time::Duration::from_secs(1));
    
        Sse::new(stream).keep_alive(
            axum::response::sse::KeepAlive::new()
                .interval(std::time::Duration::from_secs(1))
                .text(rand::thread_rng().sample_iter(&Alphanumeric).take(5).map(char::from).collect::<String>()),
        )
    }
}

