use crate::measurements::Measurements;
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use spin_sdk::http::{Method, Request, Response};
use serde::Deserialize;
use crate::collectors::common::wind_direction_name;
use crate::collectors::Downloader;

const API_URL: &str = "http://api.pioupiou.fr/v1/live/";
pub const BASE_URL: &str = "https://www.openwindmap.org/";

pub struct OpenWindMapDownloader {}

#[derive(Deserialize, Debug)]
struct MeasurementsRaw {
    data: MeasurementsRawData,
}

#[derive(Deserialize, Debug)]
struct MeasurementsRawData {
    measurements: MeasurementsRawMeasurements,
}

#[derive(Deserialize, Debug)]
struct MeasurementsRawMeasurements {
    date: String,
    wind_heading: f64,
    wind_speed_avg: f64,
    wind_speed_max: f64,
}

impl Downloader for OpenWindMapDownloader {
    fn base_url(&self) -> String {
        BASE_URL.to_owned()
    }

    async fn try_download(&self, url: &str) -> anyhow::Result<Measurements> {

        let path = url.strip_prefix(&BASE_URL)
            .ok_or_else(|| anyhow!("Invalid URL: {}", url))?;

        let vendor_id = path.split('-')
            .nth(1)
            .ok_or_else(|| anyhow!("Unexpected path: {}", path))?;

        let url = format!("{}{}", API_URL, vendor_id);
        let request = Request::builder()
            .method(Method::Get)
            .header("Accept-Charset", "utf-8")
            .uri(url)
            .build();

        let response: Response = spin_sdk::http::send(request).await?;
        let body = String::from_utf8_lossy(response.body());
        let measurement_raw: MeasurementsRaw = serde_json::from_str(&body)?;

        let update_time: DateTime<Utc> = measurement_raw.data.measurements.date.parse()?;

        let measurements = Measurements {
            update_time: Some(update_time.format("%Y-%m-%d %H:%M").to_string()),
            humidity: None,
            precipitation: None,
            pressure: None,
            temperature: None,
            wind_direction: Some(wind_direction_name(measurement_raw.data.measurements.wind_heading).to_owned()),
            wind_speed: Some(measurement_raw.data.measurements.wind_speed_avg.round() as u64),
            gusts_speed: Some(measurement_raw.data.measurements.wind_speed_max.round() as u64),
        };

        Ok(measurements)
    }
}
