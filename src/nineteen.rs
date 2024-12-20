use axum::{
    body::Body,
    extract::{Json, Path, Query, State},
    http::{Response, StatusCode},
    response::IntoResponse,
};
use rand::{
    distributions::{Alphanumeric, DistString},
    thread_rng,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use tracing::info;
use uuid::Uuid;

fn default_id() -> Uuid {
    Uuid::new_v4()
}

fn timestamp() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc::now()
}

fn default_version() -> i32 {
    1
}

#[derive(Debug, Deserialize, FromRow, Serialize)]
pub(super) struct Quote {
    #[serde(default = "default_id")]
    id: Uuid,
    author: String,
    quote: String,
    #[serde(default = "timestamp")]
    created_at: chrono::DateTime<chrono::Utc>,
    #[serde(default = "default_version")]
    version: i32,
}

#[derive(Debug, FromRow)]
pub struct PageToken {
    #[allow(dead_code)]
    id: String,
    page: i32,
}

#[derive(Debug, Serialize)]
pub(super) struct QuoteList {
    next_token: Option<String>,
    page: i32,
    quotes: Vec<Quote>,
}

#[derive(Clone, Debug)]
pub(super) struct QuoteState {
    pub(crate) pool: PgPool,
}

#[derive(Debug, Deserialize)]
pub(super) struct ListQuery {
    pub(crate) token: Option<String>,
}

async fn get_quote(pool: &PgPool, id: Uuid) -> Option<Quote> {
    let quote = sqlx::query_as::<_, Quote>("SELECT * FROM quotes WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .unwrap();
    quote
}

pub(super) async fn reset(State(state): State<QuoteState>) -> Response<String> {
    info!("Resetting quotes");
    sqlx::query("TRUNCATE TABLE quotes")
        .execute(&state.pool)
        .await
        .unwrap();
    sqlx::query("TRUNCATE TABLE quotes_pagination")
        .execute(&state.pool)
        .await
        .unwrap();
    Response::builder().status(200).body("".into()).unwrap()
}

pub(super) async fn cite(State(state): State<QuoteState>, Path(id): Path<Uuid>) -> Response<Body> {
    info!("Citing quote with id: {}", id);
    let quote = get_quote(&state.pool, id).await;
    match quote {
        Some(quote) => Json(quote).into_response(),
        None => Response::builder().status(404).body("".into()).unwrap(),
    }
}

pub(super) async fn remove(
    State(state): State<QuoteState>,
    Path(id): Path<Uuid>,
) -> Response<Body> {
    info!("Removing quote with id: {}", id);
    let quote = get_quote(&state.pool, id).await;
    match quote {
        Some(quote) => {
            sqlx::query("DELETE FROM quotes WHERE id = $1")
                .bind(id)
                .execute(&state.pool)
                .await
                .unwrap();
            info!("Successfully removed quote with ID: {id}");
            Json(quote).into_response()
        }
        None => {
            info!("Failed to remove quote with ID: {}", id);
            Response::builder().status(404).body("".into()).unwrap()
        }
    }
}

pub(super) async fn undo(
    State(state): State<QuoteState>,
    Path(id): Path<Uuid>,
    Json(new_quote): Json<Quote>,
) -> Response<Body> {
    info!("Undoing quote with ID: {id}");
    let old_quote = get_quote(&state.pool, id).await;
    match old_quote {
        Some(mut updated) => {
            info!("Undoing quote with id: {}, new_quote: {:?}", id, &new_quote);
            updated.author = new_quote.author;
            updated.quote = new_quote.quote;
            updated.version += 1;
            sqlx::query("UPDATE quotes SET author = $1, quote = $2, version = $3 WHERE id = $4")
                .bind(&updated.author)
                .bind(&updated.quote)
                .bind(&updated.version)
                .bind(id)
                .execute(&state.pool)
                .await
                .unwrap();
            Json(updated).into_response()
        }
        None => {
            info!("Failed to undo quote with ID: {}", id,);
            Response::builder().status(404).body("".into()).unwrap()
        }
    }
}

#[axum::debug_handler]
pub(super) async fn draft(
    State(state): State<QuoteState>,
    Json(new_quote): Json<Quote>,
) -> Response<Body> {
    info!("Drafting new quote: {:?}", &new_quote);
    sqlx::query(
        "INSERT INTO quotes (id, author, quote, created_at, version) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(new_quote.id)
    .bind(&new_quote.author)
    .bind(&new_quote.quote)
    .bind(&new_quote.created_at)
    .bind(new_quote.version)
    .execute(&state.pool)
    .await
    .unwrap();
    let mut response = Json(new_quote).into_response();
    *response.status_mut() = StatusCode::CREATED;
    response
}

#[axum::debug_handler]
pub(super) async fn list(
    State(state): State<QuoteState>,
    query: Query<ListQuery>,
) -> Response<Body> {
    let requested_page = match &query.token {
        Some(token) => {
            let token_info =
                sqlx::query_as::<_, PageToken>("SELECT * FROM quotes_pagination WHERE id = $1")
                    .bind(&token)
                    .fetch_optional(&state.pool)
                    .await
                    .unwrap();
            match token_info {
                Some(token_info) => token_info.page,
                None => {
                    return Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body("".into())
                        .unwrap();
                }
            }
        }
        None => 0,
    };

    info!(
        "Listing quotes with page token: {:?}, Page: {}",
        query.token,
        requested_page + 1
    );
    let offset: i32 = requested_page as i32 * 3;

    info!("Offset: {offset}");
    let quotes = sqlx::query_as::<_, Quote>(
        "SELECT * FROM quotes ORDER BY created_at ASC OFFSET $1 LIMIT 4",
    )
    .bind(offset)
    .fetch_all(&state.pool)
    .await
    .unwrap();

    let mut token = None;
    if quotes.len() == 4 {
        let token_id = Alphanumeric.sample_string(&mut thread_rng(), 16);
        sqlx::query("INSERT INTO quotes_pagination (id, page) VALUES ($1, $2)")
            .bind(&token_id)
            .bind(requested_page + 1)
            .execute(&state.pool)
            .await
            .unwrap();
        token = Some(token_id);
    }
    let list = QuoteList {
        quotes: quotes.into_iter().take(3).collect(),
        page: requested_page + 1,
        next_token: token,
    };
    Json(list).into_response()
}
