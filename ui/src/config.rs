use crate::utils::log_anyhow_error;
use anyhow::Context;
use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use leptos::leptos_dom::logging::{console_log, console_warn};
use serde_json::json;
use url::Url;

const AEMET_BASE_URL: &str = "https://www.aemet.es/";
const METEOCAT_BASE_URL: &str = "https://www.meteo.cat/";
const METEOCLIMATIC_BASE_URL: &str = "https://www.meteoclimatic.net/";

const CONFIG_ANNOTATIONS: &str = r#"
# This is your configuration file. Feel free to edit it. When you are done:
# - Click save button.
# - Copy your lesma URL.
# - Go back to the weather-data app.
# - Provide your lesma URL in the "Import Configuration" window.
# 
# URLs of the weather stations can be obtained from the address bar of your
# browser after clicking on the station in one of the following maps:
#   https://www.meteoclimatic.net/mapinfo/ESCAT
#   https://www.meteo.cat/observacions/xema
#   https://www.aemet.es/en/eltiempo/observacion/ultimosdatos
#
# Keys of the measurements can be picked from the following list:
#   location
#   update_time_utc
#   update_time_ago
#   wind_direction
#   wind_speed
#   gusts_speed
#   humidity
#   precipitation
#   temperature
#   pressure
#
"#;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ConfigStation {
    pub label: String,
    pub url: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ConfigMeasurement {
    pub label: String,
    pub key: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Config {
    pub stations: Vec<ConfigStation>,
    pub measurements: Vec<ConfigMeasurement>,
}

fn default_config() -> Config {
    let config = json!(
        {"stations": [
            {"label": "Bellmunt Sanctuary", "url": "https://www.meteoclimatic.net/perfil/ESCAT0800000008572A"},
            {"label": "Berga Sanctuary", "url": "https://www.meteo.cat/observacions/xema/dades?codi=WM"},
            {"label": "Berga Town", "url": "https://www.meteoclimatic.net/perfil/ESCAT0800000008600A"},
            {"label": "Avia Town", "url": "https://www.meteoclimatic.net/perfil/ESCAT0800000008610A"},
            {"label": "Belltall", "url": "https://www.meteoclimatic.net/perfil/ESCAT4300000043413A"},
            {"label": "Pineda Town", "url": "https://www.meteoclimatic.net/perfil/ESCAT0800000008397A"},
            {"label": "Ager Observatory", "url": "https://www.meteo.cat/observacions/xema/dades?codi=WQ"},
            {"label": "Manresa Town", "url": "https://www.aemet.es/en/eltiempo/observacion/ultimosdatos?k=cat&l=0149X"}
        ],
        "measurements": [
            {"label": "Location", "key": "location"},
            //{"label": "Update Time", "key": "update_time_utc"},
            {"label": "Update Time", "key": "update_time_ago"},
            {"label": "Wind Direction", "key": "wind_direction"},
            {"label": "Wind Speed", "key": "wind_speed"},
            {"label": "Gusts Speed", "key": "gusts_speed"},
            {"label": "Humidity", "key": "humidity"},
            {"label": "Precipitation", "key": "precipitation"},
            {"label": "Temperature", "key": "temperature"},
            {"label": "Pressure", "key": "pressure"},
        ]
        }
    );
    serde_json::to_string(&config)
        .map_err(|e| e.into())
        .and_then(|c| parse_config(&c))
        .expect("Invalid default configuration")
}

pub fn get_local_config() -> Config {
    let config: anyhow::Result<Config> = LocalStorage::get("config").map_err(|e| e.into());
    config.unwrap_or_else(|e| {
        log_anyhow_error(e.context("Failed to load local config"));
        let config = default_config();
        console_log("Default config generated");
        set_local_config(&config);
        config
    })
}

pub fn set_local_config(config: &Config) {
    LocalStorage::set("config", config).unwrap_or_else(|e| {
        let e = anyhow::Error::new(e);
        log_anyhow_error(e.context("Failed to save configuration"));
    });
}

async fn do_upload_config(config: &Config) -> anyhow::Result<String> {
    let config_str = serde_json::to_string_pretty(&config).context("Failed to serialize config")?;
    let config_annotated = CONFIG_ANNOTATIONS.to_string() + config_str.as_str();

    let url = "/pbproxy";
    let req = Request::post(url)
        .body(config_annotated)
        .context("Failed to prepare request")?;
    let resp = req.send().await.context("Failed to get response")?;

    if resp.status() == 303 {
        let id = resp.text().await.context("Failed to decode response")?;
        return Ok(id);
    }
    anyhow::bail!("Unexpected response form pbproxy: {}", resp.status());
}

pub async fn upload_config(config: &Config) -> anyhow::Result<String> {
    do_upload_config(config)
        .await
        .context("Failed to upload config")
}

pub async fn download_config(id: &str) -> anyhow::Result<Config> {
    let url = format!("/pbproxy/{}", id);
    let req = Request::get(&url);
    let resp = req.send().await.context("Failed to get response")?;

    if resp.status() == 200 {
        let config = resp.text().await.context("Failed to decode response")?;
        let config_bare = config
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .collect::<Vec<_>>()
            .join("");
        return parse_config(&config_bare);
    }
    anyhow::bail!("Unexpected response form pbproxy: {}", resp.status());
}

pub fn parse_config(config: &str) -> anyhow::Result<Config> {
    let mut config: Config = serde_json::from_str(config)?;
    config
        .stations
        .iter_mut()
        .for_each(|item| match sanitize_url(item.url.as_str()) {
            Ok(url) => item.url = url,
            Err(e) => {
                log_anyhow_error(e.context("Failed to sanitize URL"));
            }
        });
    Ok(config)
}

fn sanitize_url(url: &str) -> anyhow::Result<String> {
    // scheme and domain are case insensitive
    let url_lower = url.to_lowercase();
    if url_lower.starts_with(AEMET_BASE_URL) {
        sanitize_url_aemet(url)
    } else if url_lower.starts_with(METEOCLIMATIC_BASE_URL) {
        sanitize_url_meteoclimatic(url)
    } else if url_lower.starts_with(METEOCAT_BASE_URL) {
        sanitize_url_meteocat(url)
    } else {
        console_warn(&format!("Unsupported source: {}", url));
        Ok(url.to_string())
    }
}

fn sanitize_url_aemet(url: &str) -> anyhow::Result<String> {
    let mut url = Url::parse(url)?;

    let white_list = ["k", "l"];
    let preserved_params: Vec<_> = url
        .query_pairs()
        .filter(|(k, _)| white_list.contains(&k.as_ref()))
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();

    url.query_pairs_mut().clear();
    for (k, v) in preserved_params {
        url.query_pairs_mut().append_pair(&k, &v);
    }
    url.set_fragment(None);
    url.to_string();
    let url_string = url.to_string();
    Ok(url_string
        .strip_suffix('?')
        .unwrap_or(&url_string)
        .to_string())
}

fn sanitize_url_meteoclimatic(url: &str) -> anyhow::Result<String> {
    let mut url = Url::parse(url)?;
    url.query_pairs_mut().clear();
    url.set_fragment(None);
    let url_string = url.to_string();
    Ok(url_string
        .strip_suffix('?')
        .unwrap_or(&url_string)
        .to_string())
}

fn sanitize_url_meteocat(url: &str) -> anyhow::Result<String> {
    let mut url = Url::parse(url)?;

    let white_list = ["codi"];
    let preserved_params: Vec<_> = url
        .query_pairs()
        .filter(|(k, _)| white_list.contains(&k.as_ref()))
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();

    url.query_pairs_mut().clear();
    for (k, v) in preserved_params {
        url.query_pairs_mut().append_pair(&k, &v);
    }
    url.set_fragment(None);
    let url_string = url.to_string();
    Ok(url_string
        .strip_suffix('?')
        .unwrap_or(&url_string)
        .to_string())
}
