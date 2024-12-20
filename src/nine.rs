use std::sync::{Arc, Mutex};

use axum::{
    extract::State,
    http::{header::CONTENT_TYPE, HeaderMap, Response, StatusCode},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Clone, Copy)]
pub(super) struct MilkState {
    pub(crate) milk: f32,
    pub(crate) updated: DateTime<Utc>,
}

impl MilkState {
    pub(crate) fn construct() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            milk: 0.0,
            updated: DateTime::UNIX_EPOCH,
        }))
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub(super) struct ConversionRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    liters: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    gallons: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    litres: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pints: Option<f32>,
}

fn update_state(state: Arc<Mutex<MilkState>>) -> f32 {
    let mut state = state.lock().unwrap();
    let elapsed = Utc::now().signed_duration_since(state.updated);
    state.milk = f32::min(state.milk + elapsed.num_seconds() as f32 * 1.0, 5_f32);
    state.updated = Utc::now();
    state.milk
}

fn withdraw_milk(state: Arc<Mutex<MilkState>>) {
    let mut state = state.lock().unwrap();
    state.milk -= 1.0;
    state.updated = Utc::now();
}
fn refill_milk(state: Arc<Mutex<MilkState>>) {
    let mut state = state.lock().unwrap();
    state.milk = 5.0;
    state.updated = Utc::now();
}

pub(super) async fn milk(
    State(state): State<Arc<Mutex<MilkState>>>,
    headers: HeaderMap,
    body_text: String,
) -> Response<String> {
    let current_milk = update_state(state.clone());
    if current_milk >= 1.0 {
        withdraw_milk(state);
        info!("Milk withdrawn: {current_milk}");
        let body: Option<ConversionRequest>;
        if let Some(media_type) = headers.get(CONTENT_TYPE) {
            if media_type == "application/json" {
                body = conversion(&body_text);
                if body.is_none() {
                    // The body was invalid
                    info!("Bad Request");
                    return bad_request();
                }
            } else {
                body = None;
            }
        } else {
            body = None;
        }
        if let Some(body) = body {
            let body = serde_json::to_string(&body).unwrap();
            info!("Conversion: {body}");
            Response::builder()
                .status(StatusCode::OK)
                .body(body)
                .unwrap()
        } else {
            Response::builder()
                .status(StatusCode::OK)
                .body("Milk withdrawn\n".into())
                .unwrap()
        }
    } else {
        info!("No milk available");
        Response::builder()
            .status(StatusCode::TOO_MANY_REQUESTS)
            .body("No milk available\n".into())
            .unwrap()
    }
}

pub(super) async fn refill(State(state): State<Arc<Mutex<MilkState>>>) -> Response<String> {
    refill_milk(state);
    Response::builder()
        .status(StatusCode::OK)
        .body("".into())
        .unwrap()
}

fn bad_request() -> Response<String> {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body("".into())
        .unwrap()
}

fn conversion(body_text: &str) -> Option<ConversionRequest> {
    info!("{body_text}");
    let Ok(conversion) = serde_json::from_str::<ConversionRequest>(&body_text) else {
        return None;
    };
    const LITERS_TO_GALLONS: f32 = 0.264172;
    const GALLONS_TO_LITERS: f32 = 3.78541;
    const PINTS_TO_LITRES: f32 = 0.568261;
    const LITRES_TO_PINTS: f32 = 1.75975;
    if !(conversion.liters.is_some()
        ^ conversion.gallons.is_some()
        ^ conversion.litres.is_some()
        ^ conversion.pints.is_some())
    {
        return None;
    }
    if let Some(liters) = conversion.liters {
        return Some(ConversionRequest {
            liters: None,
            gallons: Some(liters * LITERS_TO_GALLONS),
            litres: None,
            pints: None,
        });
    } else if let Some(gallons) = conversion.gallons {
        Some(ConversionRequest {
            liters: Some(gallons * GALLONS_TO_LITERS),
            gallons: None,
            litres: None,
            pints: None,
        })
    } else if let Some(litres) = conversion.liters {
        Some(ConversionRequest {
            liters: None,
            gallons: Some(litres * LITERS_TO_GALLONS),
            litres: None,
            pints: None,
        })
    } else if let Some(pints) = conversion.pints {
        Some(ConversionRequest {
            liters: None,
            gallons: None,
            litres: Some(pints * PINTS_TO_LITRES),
            pints: None,
        })
    } else if let Some(litres) = conversion.litres {
        Some(ConversionRequest {
            liters: None,
            gallons: None,
            litres: None,
            pints: Some(litres * LITRES_TO_PINTS),
        })
    } else {
        None
    }
}
