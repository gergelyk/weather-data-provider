use crate::measurements::Measurements;
use anyhow::anyhow;
use chrono::DateTime;
use spin_sdk::http::{Method, Request, Response};
use serde::Deserialize;
use crate::collectors::common::wind_direction_name;
use crate::collectors::Downloader;

pub const BASE_URL: &str = "https://www.weatherlink.com/";

pub struct WeatherlinkDownloader {}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct MeasurementsRaw {
    windDirection: i64,
    barometerUnits: String,
    windUnits: String,
    rainUnits: String,
    tempUnits: String,
    temperature: String,
    wind: String,
    gust: String,
    humidity: String,
    rain: String,
    barometer: String,
    lastReceived: u64,
}

impl Downloader for WeatherlinkDownloader {
    fn base_url(&self) -> String {
        BASE_URL.to_owned()
    }

    async fn try_download(&self, url: &str) -> anyhow::Result<Measurements> {

        let prefix = format!("{}embeddablePage/show/", BASE_URL);
        let path = url.strip_prefix(&prefix)
            .ok_or_else(|| anyhow!("Invalid URL: {}", url))?;

        let vendor_id = path.split('/')
            .next()
            .ok_or_else(|| anyhow!("Unexpected path: {}", path))?;

        let url = format!("{}embeddablePage/getData/{}", BASE_URL, vendor_id);
        let request = Request::builder()
            .method(Method::Get)
            .header("Accept-Charset", "utf-8")
            .uri(url)
            .build();

        let response: Response = spin_sdk::http::send(request).await?;
        let body = String::from_utf8_lossy(response.body());
        let measurement_raw: MeasurementsRaw = serde_json::from_str(&body)?;

        if !["mb", "hPa"].contains(&measurement_raw.barometerUnits.as_str()) {
            anyhow::bail!("Unsupported barometer units: {}", measurement_raw.barometerUnits);
        }
        if measurement_raw.windUnits != "km/h" {
            anyhow::bail!("Unsupported wind units: {}", measurement_raw.windUnits);
        }
        if measurement_raw.rainUnits != "mm" {
            anyhow::bail!("Unsupported rain units: {}", measurement_raw.rainUnits);
        }
        if measurement_raw.tempUnits != "&deg;C" {
            anyhow::bail!("Unsupported temperature units: {}", measurement_raw.tempUnits);
        }
        let update_time = DateTime::from_timestamp(
            measurement_raw.lastReceived as i64 / 1000,
            (measurement_raw.lastReceived % 1000 * 1_000_000) as u32,
        );

        let measurements = Measurements {
            update_time: update_time.map(|t| t.format("%Y-%m-%d %H:%M").to_string()),
            humidity: Some(measurement_raw.humidity.parse()?),
            precipitation: Some(measurement_raw.rain.parse()?),
            pressure: Some(measurement_raw.barometer.parse::<f64>()?.round() as u64), // 1 mb = 1 hPa
            temperature: Some(measurement_raw.temperature.parse()?),
            wind_direction: Some(wind_direction_name(measurement_raw.windDirection as f64).to_owned()),
            wind_speed: Some(measurement_raw.wind.parse()?),
            gusts_speed: Some(measurement_raw.gust.parse()?),
        };

        Ok(measurements)
    }
}
