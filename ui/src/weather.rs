use crate::config::Config;
use crate::utils::log_anyhow_error;
use anyhow::Context;
use chrono::NaiveDateTime;
use gloo_net::http::Request;
use serde::Deserialize;
use std::collections::HashMap;

const API_TOKEN: &str = env!("SPIN_VARIABLE_API_TOKEN");

const KEY_UPDATE_TIME: &str = "update_time";
const KEY_UPDATE_TIME_UTC: &str = "update_time_utc";
const KEY_UPDATE_TIME_AGO: &str = "update_time_ago";
const KEY_LOCATION: &str = "location";

#[derive(Clone, Debug)]
pub enum CellValue {
    Link(String, String),
    Text(String),
    NotAvailable,
}

pub type Headers = Vec<(String, String)>;
pub type Measurements = Vec<Vec<CellValue>>;
pub type WeatherData = (Headers, Measurements);

#[derive(Deserialize, Debug)]
struct WeatherDataRaw {
    pub units: HashMap<String, String>,
    pub measurements: Vec<HashMap<String, Option<serde_json::Value>>>,
}

fn time_delta(now: NaiveDateTime, then: NaiveDateTime) -> String {
    let delta = now - then;
    let days = delta.num_days();
    let hours = delta.num_hours() % 24;
    let minutes = delta.num_minutes() % 60;
    if days > 0 {
        let hours_rounded = if minutes > 30 { hours + 1 } else { hours };
        format!("{}d {}h", days, hours_rounded)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

fn convert_utc_to_ago(units: &mut HashMap<String, String>, weather_data: &mut WeatherDataRaw) {
    if let Some(update_time_unit) = units.get(KEY_UPDATE_TIME).cloned() {
        if update_time_unit == "UTC" {
            units.remove(KEY_UPDATE_TIME);
            units.insert(KEY_UPDATE_TIME_UTC.to_owned(), update_time_unit.clone());
            units.insert(KEY_UPDATE_TIME_AGO.to_owned(), "Ago".to_owned());

            let now_utc = chrono::Utc::now();
            for measure in weather_data.measurements.iter_mut() {
                let mut update_time_ago: Option<serde_json::Value> = None;
                let update_time_utc = measure.get(KEY_UPDATE_TIME);

                if let Some(update_time_utc) = update_time_utc {
                    update_time_ago = update_time_utc
                        .as_ref()
                        .and_then(|t| t.as_str())
                        .and_then(|t| NaiveDateTime::parse_from_str(t, "%Y-%m-%d %H:%M").ok())
                        .map(|t| time_delta(now_utc.naive_utc(), t))
                        .and_then(|t| serde_json::to_value(&t).ok());

                    measure.insert(KEY_UPDATE_TIME_UTC.to_owned(), update_time_utc.clone());
                }
                measure.insert(KEY_UPDATE_TIME_AGO.to_owned(), update_time_ago);
            }
        }
    }
}

pub async fn get_weather_data(config: Config) -> anyhow::Result<WeatherData> {
    let api_url = format!("/api/v1?token={}", API_TOKEN);
    let sources = config
        .stations
        .iter()
        .map(|item| item.url.clone())
        .collect::<Vec<_>>();
    let sources = serde_json::to_string(&sources)
        .context("Failed to serialize sources data")?
        .to_string();
    let resp = Request::post(&api_url).body(&sources)?.send().await?;

    let text = resp.text().await?;
    if !resp.ok() {
        log_anyhow_error(anyhow::anyhow!("API response: {}", text));
        anyhow::bail!("HTTP error from the weather-data API: {}", resp.status());
    }

    let mut weather_data_raw: WeatherDataRaw =
        serde_json::from_str(&text).context("Failed to parse weather data response JSON")?;
    let mut units = std::mem::take(&mut weather_data_raw.units);

    convert_utc_to_ago(&mut units, &mut weather_data_raw);

    let mut measurement_raw: Vec<HashMap<String, CellValue>> = weather_data_raw
        .measurements
        .iter()
        .map(|measurement| {
            measurement
                .iter()
                .map(|(key, value)| {
                    let value = match value {
                        Some(v) => match v {
                            serde_json::Value::String(s) => CellValue::Text(s.clone()),
                            _ => CellValue::Text(v.to_string()),
                        },
                        None => CellValue::NotAvailable,
                    };
                    (key.clone(), value)
                })
                .collect()
        })
        .collect();

    measurement_raw
        .iter_mut()
        .zip(config.stations)
        .for_each(|(row_values, row_config)| {
            let location = CellValue::Link(row_config.label.clone(), row_config.url.clone());
            row_values.insert(KEY_LOCATION.to_owned(), location);
        });

    let headers = config
        .measurements
        .iter()
        .map(|measurement| {
            let title = measurement.label.clone();
            let unit = units.get(&measurement.key).cloned().unwrap_or_default();
            (title, unit)
        })
        .collect::<Vec<_>>();

    let measurements = measurement_raw
        .iter()
        .map(|row| {
            config
                .measurements
                .iter()
                .map(|column_config| {
                    row.get(&column_config.key)
                        .cloned()
                        .unwrap_or(CellValue::NotAvailable)
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    Ok((headers, measurements))
}
