use axum::{extract::State, http::Response};
use sqlx::PgPool;

pub(super) async fn reset(State(state): State<PgPool>) -> Response<String> {
    Response::builder().status(200).body("".into()).unwrap()
}
