use crate::downloader::Downloader;
use crate::measurements::Measurements;
use anyhow::{anyhow, Context};
use chrono::NaiveDateTime;
use scraper::{Html, Selector};
use spin_sdk::http::{Method, Request, Response};

pub const BASE_URL: &str = "https://www.meteo.cat/";

pub struct MeteocatDownloader {}

fn wind_direction_name(degrees: f64) -> &'static str {
    const DIRECTIONS: [&str; 16] = [
        "N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE", "S", "SSW", "SW", "WSW", "W", "WNW",
        "NW", "NNW",
    ];
    // Normalize degrees to [0, 360)
    let deg = degrees.rem_euclid(360.0);
    // Each sector is 22.5 degrees
    let idx = ((deg + 11.25) / 22.5).floor() as usize % 16;
    DIRECTIONS[idx]
}

fn parse_selector(selector: &str) -> anyhow::Result<Selector> {
    Selector::parse(selector).map_err(|e| anyhow!(e.to_string()))
}

impl Downloader for MeteocatDownloader {
    fn base_url(&self) -> String {
        BASE_URL.to_owned()
    }

    async fn try_download(&self, url: &str) -> anyhow::Result<Measurements> {
        let request = Request::builder()
            .method(Method::Get)
            .header("Accept-Charset", "utf-8")
            .uri(url)
            .build();

        let response: Response = spin_sdk::http::send(request).await?;
        let body = String::from_utf8_lossy(response.body());
        let document = Html::parse_document(&body);

        let table_selector = parse_selector("table.tblperiode")?;
        let row_selector = parse_selector("tr")?;
        let header_selector = parse_selector("th")?;
        let cell_selector = parse_selector("td")?;
        let span_selector = parse_selector("span")?;
        let date_selector = parse_selector("input#datepicker")?;

        let table = document
            .select(&table_selector)
            .next()
            .ok_or(anyhow!("Table not found"))?;
        let mut rows = table.select(&row_selector);
        let row0 = rows.next().ok_or(anyhow!("Header not found"))?;

        let titles = row0
            .select(&header_selector)
            .map(|th| {
                th.select(&span_selector)
                    .next()
                    .and_then(|s| s.attr("title"))
                    .map(|t| t.trim())
            })
            .collect::<Option<Vec<_>>>()
            .ok_or(anyhow!("Titles not found"))?;

        let mut last_time: Option<String> = None;
        let mut last_valid_row: Option<Vec<String>> = None;

        for row in rows {
            let time_range = row
                .select(&header_selector)
                .next()
                .ok_or(anyhow!("Time cell not found"))?;
            let time_range_str = time_range.text().collect::<String>().trim().to_string();
            let time_boundaries = time_range_str.split(" - ").collect::<Vec<&str>>();
            let time_end = time_boundaries
                .last()
                .ok_or(anyhow!("Time end not found"))?;

            let cells = row
                .select(&cell_selector)
                .map(|td| {
                    let cell_text = td.text().collect::<String>().trim().to_string();
                    cell_text
                })
                .collect::<Vec<String>>();
            let all_not_available = cells.iter().all(|c| c == "(s/d)");
            if !all_not_available {
                last_time = Some(time_end.to_string());
                last_valid_row = Some(cells);
            }
        }

        let time = last_time.ok_or(anyhow!("No valid times"))?;
        let measurements = last_valid_row.ok_or(anyhow!("No valid rows"))?;

        let date_tag = document
            .select(&date_selector)
            .next()
            .ok_or(anyhow!("Date tag not found"))?;
        let date_str = date_tag
            .attr("value")
            .ok_or(anyhow!("Date not found"))?
            .trim()
            .to_string();
        let update_time =
            NaiveDateTime::parse_from_str(&format!("{} {}", date_str, time), "%d.%m.%Y %H:%M")
                .context("Date parsing failed")?;

        let get_measurement = |name: &str| {
            let index = titles.iter().skip(1).position(|&s| s == name);
            if let Some(idx) = index {
                return measurements.get(idx);
            }
            None
        };

        let humidity =
            get_measurement("Humitat relativa mitjana (%)").and_then(|v| v.parse::<u64>().ok());
        let precipitation =
            get_measurement("Precipitació (mm)").and_then(|v| v.parse::<f64>().ok());
        let pressure = get_measurement("Pressió atmosfèrica mitjana (hPa)")
            .and_then(|v| v.parse::<f64>().ok());
        let temperature =
            get_measurement("Temperatura mitjana (°C)").and_then(|v| v.parse::<f64>().ok());
        let wind_direction_degrees = get_measurement("Direcció mitjana del vent (graus)")
            .and_then(|v| v.parse::<f64>().ok());
        let wind_speed = get_measurement("Velocitat mitjana del vent (km/h)")
            .and_then(|v| v.parse::<f64>().ok());
        let gusts_speed =
            get_measurement("Ratxa màxima del vent (km/h)").and_then(|v| v.parse::<f64>().ok());

        let wind_direction = wind_direction_degrees.map(wind_direction_name);

        let measurements = Measurements {
            update_time: Some(update_time.format("%Y-%m-%d %H:%M").to_string()),
            humidity,
            precipitation,
            pressure: pressure.map(|p| p.round() as u64),
            temperature,
            wind_direction: wind_direction.map(|s| s.to_owned()),
            wind_speed: wind_speed.map(|p| p.round() as u64),
            gusts_speed: gusts_speed.map(|p| p.round() as u64),
        };

        Ok(measurements)
    }
}
