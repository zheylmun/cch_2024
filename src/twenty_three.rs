use core::str;

use axum::{
    body::Body,
    extract::{Multipart, Path},
    http::Response,
};
use serde::Deserialize;
use tracing::info;

pub(super) async fn star() -> &'static str {
    "<div class=\"lit\" id=\"star\"></div>"
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum PresentColor {
    Red,
    Blue,
    Purple,
}

pub(super) async fn present(Path(color): Path<String>) -> Response<Body> {
    let color = html_escape::encode_safe(&color).to_string();
    info!("Present changing to color: {:?}", color);
    let color = match color.as_str() {
        "red" => PresentColor::Red,
        "blue" => PresentColor::Blue,
        "purple" => PresentColor::Purple,
        _ => {
            return teapot();
        }
    };

    let (current, next) = match color {
        PresentColor::Red => ("red", "blue"),
        PresentColor::Blue => ("blue", "purple"),
        PresentColor::Purple => ("purple", "red"),
    };
    Response::builder()
        .status(200)
        .body(
            format!(
            "<div class=\"present {current}\" hx-get=\"/23/present/{next}\" hx-swap=\"outerHTML\">
        <div class=\"ribbon\"></div>
        <div class=\"ribbon\"></div>
        <div class=\"ribbon\"></div>
        <div class=\"ribbon\"></div>
        </div>"
        )
            .into(),
        )
        .unwrap()
}

enum State {
    On,
    Off,
}
pub(super) async fn ornament(Path((state, number)): Path<(String, String)>) -> Response<Body> {
    info!("Updating ornament{number} with state: {state}");
    let state = html_escape::encode_safe(&state).to_string();
    let number = html_escape::encode_safe(&number).to_string();
    let state = match state.as_str() {
        "on" => Ok(State::On),
        "off" => Ok(State::Off),
        _ => Err("Nope"),
    };

    if state.is_err() {
        return teapot();
    }
    let state = state.unwrap();

    let (class, next) = match state {
        State::On => (" on", "off"),
        State::Off => ("", "on"),
    };
    Response::builder()
        .status(200)
        .body(format!("<div class=\"ornament{class}\" id=\"ornament{number}\" hx-trigger=\"load delay:2s once\" hx-get=\"/23/ornament/{next}/{number}\" hx-swap=\"outerHTML\"></div>").into()).unwrap()
}

fn teapot() -> Response<Body> {
    Response::builder()
        .status(418)
        .body("I'm a teapot".into())
        .unwrap()
}

#[derive(Debug, Deserialize)]
struct Lockfile {
    package: Vec<Package>,
}

#[derive(Debug, Deserialize)]
struct Package {
    checksum: Option<String>,
}

pub(super) async fn lockfile(mut multipart: Multipart) -> Response<Body> {
    let mut lock_file = None;

    while let Some(field) = multipart.next_field().await.unwrap_or_default() {
        if field.name().unwrap().to_string() == "lockfile" {
            let bytes = field.bytes().await.unwrap();
            let lockfile_str = str::from_utf8(&bytes).unwrap();
            let parse_result: Result<Lockfile, toml::de::Error> = toml::from_str(lockfile_str);
            if parse_result.is_err() {
                let error = parse_result.err().unwrap();
                info!("{error}");
                return Response::builder().status(400).body("".into()).unwrap();
            } else {
                lock_file = Some(parse_result.unwrap());
            }
        }
    }
    if lock_file.is_none() {
        return Response::builder().status(400).body("".into()).unwrap();
    }
    let lock_file = lock_file.unwrap();
    let mut results = String::new();
    for package in lock_file.package {
        if let Some(checksum) = package.checksum {
            let bytes = match hex::decode(checksum) {
                Ok(bytes) => bytes,
                Err(_) => {
                    return Response::builder().status(422).body("".into()).unwrap();
                }
            };
            if bytes.len() < 5 {
                return Response::builder().status(422).body("".into()).unwrap();
            }
            let color = format!("{:02x}{:02x}{:02x}", bytes[0], bytes[1], bytes[2]);
            let top = bytes[3] as u8;
            let left = bytes[4] as u8;
            results.push_str(&format!(
                "<div style=\"background-color:#{color};top:{top}px;left:{left}px;\"></div>\n"
            ));
        }
    }

    Response::builder()
        .status(200)
        .body(results.into())
        .unwrap()
}
