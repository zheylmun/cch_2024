mod five;
mod minus_one;
mod nine;
mod sixteen;
mod twelve;
mod two;

use axum::{
    routing::{get, post},
    Router,
};

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/", get(minus_one::hello_bird))
        .route("/-1/seek", get(minus_one::seek_redirect))
        .route("/2/dest", get(two::egregious_encryption))
        .route("/2/key", get(two::going_the_other_way))
        .route("/2/v6/dest", get(two::v6_dest))
        .route("/2/v6/key", get(two::v6_key))
        .route("/5/manifest", post(five::manifest))
        .route("/9/milk", post(nine::milk))
        .route("/9/refill", post(nine::refill))
        .with_state(nine::MilkState::construct())
        .route("/12/board", get(twelve::board_state))
        .route("/12/reset", post(twelve::reset_board))
        .route("/12/place/:team/:column", post(twelve::place))
        .route("/12/random-board", get(twelve::random_board))
        .with_state(twelve::AppState::construct())
        .route("/16/wrap", post(sixteen::wrap))
        .route("/16/unwrap", get(sixteen::unwrap))
        .route("/16/decode", post(sixteen::decode_token));
    Ok(router.into())
}
