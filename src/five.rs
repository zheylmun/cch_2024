use axum::{
    http::{header::CONTENT_TYPE, HeaderMap, StatusCode},
    response::Response,
};
use cargo_manifest::Manifest;

#[derive(serde::Deserialize)]
struct Metadata {
    #[serde(default)]
    orders: Vec<Order>,
}

#[serde_with::serde_as]
#[derive(serde::Deserialize)]
struct Order {
    item: String,
    #[serde_as(deserialize_as = "serde_with::DefaultOnError")]
    #[serde(default)]
    quantity: Option<u32>,
}

pub(super) async fn manifest(headers: HeaderMap, body: String) -> Response {
    let Some(media_type) = headers.get(CONTENT_TYPE) else {
        // If the Content-Type header is missing, return 415 Unsupported Media Type right away
        return Response::builder()
            .status(StatusCode::UNSUPPORTED_MEDIA_TYPE)
            .body("".into())
            .unwrap();
    };

    log::info!("Content-Type: {:?}", media_type);
    log::info!("Body: {}", body);
    let parsed_manifest: Manifest;
    match media_type.to_str() {
        Ok(media_type) => match media_type {
            "application/toml" => match toml::from_str::<Manifest>(&body) {
                Ok(manifest) => parsed_manifest = manifest,
                Err(e) => {
                    log::error!("Error parsing TOML: {:?}", e);
                    return invalid_manifest();
                }
            },
            "application/json" => match serde_json::from_str::<Manifest>(&body) {
                Ok(manifest) => {
                    parsed_manifest = manifest;
                }
                Err(_) => {
                    return invalid_manifest();
                }
            },
            "application/yaml" => match serde_yaml::from_str::<Manifest>(&body) {
                Ok(manifest) => {
                    parsed_manifest = manifest;
                }
                Err(_) => {
                    return invalid_manifest();
                }
            },
            _ => {
                return unsupported_media_type();
            }
        },
        Err(_) => return unsupported_media_type(),
    }

    let Some(package) = parsed_manifest.package else {
        return Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body("".into())
            .unwrap();
    };

    let Some(keywords) = package.keywords else {
        return no_magic();
    };
    if !keywords
        .as_local()
        .unwrap()
        .contains(&"Christmas 2024".to_string())
    {
        return no_magic();
    }
    let Some(metadata) = package.metadata else {
        return Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body("".into())
            .unwrap();
    };

    let metadata_string = toml::to_string(&metadata).unwrap();
    let parsed_orders = toml::from_str::<Metadata>(&metadata_string).unwrap();
    let mut valid_response = false;
    let mut response = String::new();

    for order in parsed_orders.orders {
        if let Some(quantity) = order.quantity {
            response.push_str(&format!("{}: {}\n", order.item, quantity));
            valid_response = true;
        }
    }
    response = response.trim().to_string();
    if valid_response {
        response = response.trim().to_string();

        Response::builder()
            .status(StatusCode::OK)
            .body(response.into())
            .unwrap()
    } else {
        return Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body("".into())
            .unwrap();
    }
}

fn invalid_manifest() -> Response {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body("Invalid manifest".into())
        .unwrap()
}

fn unsupported_media_type() -> Response {
    Response::builder()
        .status(StatusCode::UNSUPPORTED_MEDIA_TYPE)
        .body("".into())
        .unwrap()
}

fn no_magic() -> Response {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .body("Magic keyword not provided".into())
        .unwrap()
}
