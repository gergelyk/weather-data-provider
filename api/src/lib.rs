mod collectors;
mod measurements;

use crate::measurements::Measurements;
use collectors::{Downloader, AemetDownloader, MeteocatDownloader, MeteoclimaticDownloader, WeatherlinkDownloader};
use futures::stream::{self, StreamExt};
use measurements::get_units;
use serde_json::json;
use spin_sdk::http::{IntoResponse, Request, Response};
use spin_sdk::{http_component, key_value::Store};
use std::collections::HashMap;

const MAX_NUMBER_OF_MEASUREMENTS: usize = 50;

// important only when smaller than MAX_NUMBER_OF_MEASUREMENTS
const MAX_CONCURRENT_DOWNLOADS: usize = 100;

fn log_req_info(req: &Request) -> anyhow::Result<()> {
    let client_addr: &str = req
        .header("spin-client-addr")
        .map(|v| v.as_str().unwrap_or("?!"))
        .unwrap_or("?");

    let full_url: &str = req
        .header("spin-full-url")
        .map(|v| v.as_str().unwrap_or("?!"))
        .unwrap_or("?");

    let client_addr = client_addr.split(":").next().unwrap_or(client_addr);

    let store = Store::open("stats")?;
    let zero = "0".as_bytes();
    let count_bytes = store.get(client_addr)?.unwrap_or(zero.to_vec());
    let mut count = String::from_utf8_lossy(&count_bytes)
        .parse::<u64>()
        .unwrap_or(0);
    count += 1;
    store.set(client_addr, count.to_string().as_bytes())?;

    log::info!("{}#{} {} {}", client_addr, count, req.method(), full_url);
    Ok(())
}

fn plain_text_resp(status: u16, message: &str) -> Response {
    Response::builder()
        .status(status)
        .header("content-type", "text/plain")
        .body(message)
        .build()
}

fn check_token(req: &Request) -> anyhow::Result<Option<Response>> {
    let query_string = req.query();
    let query_vector = querystring::querify(query_string);
    let query: HashMap<_, _> = query_vector.into_iter().collect();

    let expected_token = spin_sdk::variables::get("api_token")?;

    if let Some(token) = query.get("token") {
        if token != &expected_token {
            log::error!("Invalid token: {}", token);
            return Ok(Some(plain_text_resp(400, "Invalid token")));
        }
    } else {
        log::error!("Missing token");
        return Ok(Some(plain_text_resp(400, "Missing token")));
    }

    Ok(None)
}

async fn dispatch(url: &str) -> Measurements {
    // scheme and domain are case insensitive
    let url_lower = url.to_lowercase();

    let aemet = AemetDownloader {};
    let meteocat = MeteocatDownloader {};
    let meteoclimatic = MeteoclimaticDownloader {};
    let weatherlink = WeatherlinkDownloader {};

    if url_lower.starts_with(&aemet.base_url()) {
        aemet.download(url).await
    } else if url_lower.starts_with(&meteocat.base_url()) {
        meteocat.download(url).await
    } else if url_lower.starts_with(&meteoclimatic.base_url()) {
        meteoclimatic.download(url).await
    } else if url_lower.starts_with(&weatherlink.base_url()) {
        weatherlink.download(url).await
    } else {
        log::warn!("Unsupported station URL: {}", url);
        Measurements::default()
    }
}

async fn handle_post(req: &Request) -> anyhow::Result<Response> {
    if let Some(resp) = check_token(req)? {
        return Ok(resp);
    };

    let body_bytes = req.body();
    let urls = match serde_json::from_slice::<Vec<String>>(body_bytes) {
        Ok(v) => v,
        Err(e) => {
            log::error!("Invalid configuration data: {}", e);
            return Ok(plain_text_resp(
                400,
                &format!("Invalid configuration data: {}", e),
            ));
        }
    };

    if urls.len() > MAX_NUMBER_OF_MEASUREMENTS {
        log::error!(
            "Too many measurements requested: {} (max: {})",
            urls.len(),
            MAX_NUMBER_OF_MEASUREMENTS
        );
        return Ok(plain_text_resp(400, "Too many measurements requested at once"));
    }

    let measurements = stream::iter(urls)
        .map(|url| async move { dispatch(url.as_str()).await })
        .buffered(MAX_CONCURRENT_DOWNLOADS)
        .collect::<Vec<_>>()
        .await;

    let data = json!({
        "measurements": measurements,
        "units": get_units(),
    });

    Ok(Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(data.to_string())
        .build())
}

#[http_component]
async fn handle_weather_data_provider(req: Request) -> anyhow::Result<impl IntoResponse> {
    simple_logger::init_with_level(log::Level::Info)?;
    log_req_info(&req)?;

    match req.method() {
        spin_sdk::http::Method::Get => {
            let app_name = env!("CARGO_PKG_NAME");
            let app_version = env!("CARGO_PKG_VERSION");
            Ok(plain_text_resp(
                200,
                &format!("Hello from {app_name} v{app_version}"),
            ))
        }
        spin_sdk::http::Method::Post => handle_post(&req).await,
        _ => Ok(plain_text_resp(405, "Method not allowed")),
    }
}
