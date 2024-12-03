use axum::{http::StatusCode, response::Response};

pub(super) async fn hello_bird() -> &'static str {
    "Hello, bird!"
}

pub(super) async fn seek_redirect() -> Response {
    Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", "https://www.youtube.com/watch?v=9Gc4QTqslN4")
        .body("".into())
        .unwrap()
}
