use std::collections::HashSet;

use axum::{
    extract::Json,
    http::{
        header::{COOKIE, SET_COOKIE},
        HeaderMap, Response, StatusCode,
    },
};
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::error;

const ENCODING_SECRET: &str = "!Such_S3cret_Much_S3cure!";

const SANTA_KEY: &str = include_str!("../assets/day16_santa_public_key.pem");

#[derive(Debug, Deserialize, Serialize)]
struct Claim {
    contents: serde_json::Value,
    exp: usize,
}

pub(super) async fn wrap(Json(body): Json<serde_json::Value>) -> Response<String> {
    let encoding_key = EncodingKey::from_secret(ENCODING_SECRET.as_ref());
    let claims = Claim {
        contents: body,
        exp: Utc::now().timestamp() as usize + 6000,
    };
    let token = encode(&Header::default(), &claims, &encoding_key).unwrap();
    let test = decode::<Claim>(
        &token,
        &DecodingKey::from_secret(ENCODING_SECRET.as_ref()),
        &Validation::new(jsonwebtoken::Algorithm::HS256),
    );
    if let Err(test) = test {
        error!("{test}")
    }
    let cookie = format!("gift={token}");
    Response::builder()
        .status(StatusCode::OK)
        .header(SET_COOKIE, cookie)
        .body("".into())
        .unwrap()
}

pub(super) async fn unwrap(headers: HeaderMap) -> Response<String> {
    let Some(cookie) = headers.get(COOKIE) else {
        // If the Cookie header is missing, return 400 Bad Request right away
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("".into())
            .unwrap();
    };
    let mut cookie = cookie.to_str().unwrap();
    if !cookie.starts_with("gift=") {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("".into())
            .unwrap();
    } else {
        (_, cookie) = cookie.split_at(5);
    }

    let decoding_key = DecodingKey::from_secret(ENCODING_SECRET.as_ref());
    let token = decode::<Claim>(
        &cookie,
        &decoding_key,
        &Validation::new(jsonwebtoken::Algorithm::HS256),
    )
    .unwrap();
    let serialized_claims = serde_json::to_string(&token.claims.contents).unwrap();
    Response::builder()
        .status(StatusCode::OK)
        .body(serialized_claims)
        .unwrap()
}

pub(super) async fn decode_token(jwt: String) -> Response<String> {
    let decoding_key = DecodingKey::from_rsa_pem(SANTA_KEY.as_ref()).unwrap();
    let mut validation = Validation::new(Algorithm::RS256);

    validation.required_spec_claims = HashSet::new();
    validation.algorithms = vec![Algorithm::RS256, Algorithm::RS512];
    let token = decode::<Value>(&jwt, &decoding_key, &validation);
    match token {
        Ok(token) => {
            let serialized_claims = serde_json::to_string(&token.claims).unwrap();
            Response::builder()
                .status(StatusCode::OK)
                .body(serialized_claims)
                .unwrap()
        }
        Err(e) => {
            error!("Token Rejected: {jwt}");
            error!("{e}");

            match e.kind() {
                jsonwebtoken::errors::ErrorKind::Json(_) => Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body("".into())
                    .unwrap(),
                jsonwebtoken::errors::ErrorKind::InvalidSignature => Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body("".into())
                    .unwrap(),
                _ => Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body("".into())
                    .unwrap(),
            }
        }
    }
}
