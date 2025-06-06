use crate::collectors::Downloader;
use crate::measurements::Measurements;
use anyhow::{anyhow, Context};
use chrono::NaiveDateTime;
use chrono::TimeZone;
use chrono_tz::Europe::Madrid;
use scraper::{Html, Selector};
use spin_sdk::http::{Method, Request, Response};

pub const BASE_URL: &str = "https://www.aemet.es/";

pub struct AemetDownloader {}

fn parse_selector(selector: &str) -> anyhow::Result<Selector> {
    Selector::parse(selector).map_err(|e| anyhow!(e.to_string()))
}

impl Downloader for AemetDownloader {
    fn base_url(&self) -> String {
        BASE_URL.to_owned()
    }

    async fn try_download(&self, url: &str) -> anyhow::Result<Measurements> {
        let url = format!("{}&w=0&datos=det", url);
        let request = Request::builder()
            .method(Method::Get)
            .header("Accept-Charset", "utf-8")
            .uri(url)
            .build();

        let response: Response = spin_sdk::http::send(request).await?;
        let body = String::from_utf8_lossy(response.body());
        let document = Html::parse_document(&body);

        let table_selector = parse_selector("table#table")?;
        let row_selector = parse_selector("tr")?;
        let header_selector = parse_selector("th")?;
        let cell_selector = parse_selector("td")?;

        let table = document
            .select(&table_selector)
            .next()
            .ok_or(anyhow!("Table not found"))?;
        let mut rows = table.select(&row_selector);
        let row0 = rows.next().ok_or(anyhow!("Header not found"))?;

        let titles = row0
            .select(&header_selector)
            .map(|th| {
                let title = th.attr("title").or_else(|| th.attr("abbr"));
                title.map(|t| t.trim())
            })
            .collect::<Option<Vec<_>>>()
            .ok_or(anyhow!("Titles not found"))?;

        let mut last_valid_row: Option<Vec<String>> = None;

        for row in rows {
            let cells = row
                .select(&cell_selector)
                .map(|td| {
                    let cell_text = td.text().collect::<String>().trim().to_string();
                    cell_text
                })
                .collect::<Vec<String>>();

            let all_not_available = cells.iter().skip(1).all(|c| c.is_empty());
            if !all_not_available {
                last_valid_row = Some(cells);
                break;
            }
        }

        let measurements = last_valid_row.ok_or(anyhow!("No valid rows"))?;
        let get_measurement = |name: &str| {
            let index = titles.iter().position(|&s| s == name);
            if let Some(idx) = index {
                return measurements.get(idx);
            }
            None
        };

        let update_time_str =
            get_measurement("Fecha y hora oficial").ok_or(anyhow!("Timestamp not found"))?;
        let humidity = get_measurement("Humidity (%)").and_then(|v| v.parse::<u64>().ok());
        let precipitation =
            get_measurement("Precipitation (mm)").and_then(|v| v.parse::<f64>().ok());
        let pressure = get_measurement("Pressure (hPa)").and_then(|v| v.parse::<f64>().ok());
        let temperature = get_measurement("Temperature (°C)").and_then(|v| v.parse::<f64>().ok());
        let wind_direction_code = get_measurement("Wind direction");
        let wind_speed = get_measurement("Wind speed (km/h)").and_then(|v| v.parse::<u64>().ok());
        let gusts_speed = get_measurement("Gust (km/h)").and_then(|v| v.parse::<u64>().ok());

        let update_time_native = NaiveDateTime::parse_from_str(update_time_str, "%d/%m/%Y %H:%M")
            .context("Timestamp parsing failed")?;

        // aeromet.es covers only Spain, so we always use Madrid timezone
        let update_time = Madrid
            .from_local_datetime(&update_time_native)
            .map(|dt| dt.naive_utc())
            .single();

        let wind_direction = wind_direction_code.map(|c| {
            c.split('-')
                .skip(1)
                .map(|w| w.get(0..1).unwrap_or_default())
                .collect::<Vec<&str>>()
                .join("")
                .to_uppercase()
        });

        let measurements = Measurements {
            update_time: update_time.map(|t| t.format("%Y-%m-%d %H:%M").to_string()),
            humidity,
            precipitation,
            pressure: pressure.map(|p| p.round() as u64),
            temperature,
            wind_direction: wind_direction.map(|s| s.to_owned()),
            wind_speed,
            gusts_speed,
        };

        Ok(measurements)
    }
}
