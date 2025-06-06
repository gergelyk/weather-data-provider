use crate::collectors::Downloader;
use crate::measurements::Measurements;
use regex::Regex;
use spin_sdk::http::{Method, Request, Response};
use std::collections::HashMap;

pub const BASE_URL: &str = "https://www.meteoclimatic.net/";

pub struct MeteoclimaticDownloader {}

fn parse_reading(reading: &str, unit: &str, name: &str) -> anyhow::Result<String> {
    let re = Regex::new([r#"([-\.\d]+) *"#, unit].concat().as_str())?;
    let reading = re
        .captures(reading)
        .ok_or_else(|| anyhow::anyhow!("Failed to capture {}", name))?;

    let reading = reading
        .get(1)
        .ok_or_else(|| anyhow::anyhow!("Failed to extract {}", name))?;

    Ok(reading.as_str().to_string())
}

impl Downloader for MeteoclimaticDownloader {
    fn base_url(&self) -> String {
        BASE_URL.to_owned()
    }

    async fn try_download(&self, url: &str) -> anyhow::Result<Measurements> {
        let request = Request::builder()
            .method(Method::Get)
            //.header("Accept-Charset", "utf-8") // server ignores this anyway?
            .uri(url)
            .build();

        let response: Response = spin_sdk::http::send(request).await?;
        //let body = String::from_utf8_lossy(response.body());

        // encoding reported in: <meta http-equiv="Content-Type" content="text/html; charset=
        let (body, _, _) = encoding_rs::ISO_8859_15.decode(response.body());

        let re = Regex::new(r#"class="titolet" *>(?<title>[^<]+)"#)?;
        let titles: Vec<String> = re
            .captures_iter(&body)
            .filter_map(|caps| {
                caps.name("title")
                    .map(|m| m.as_str().to_string().trim().to_string())
            })
            .collect();

        let re = Regex::new(r#"class="dadesactuals" *>(?<reading>[^<]+)"#)?;
        let readings: Vec<String> = re
            .captures_iter(&body)
            .filter_map(|caps| {
                caps.name("reading")
                    .map(|m| m.as_str().to_string().trim().to_string())
            })
            .collect();

        let dict: HashMap<_, _> = titles.into_iter().zip(readings.into_iter()).collect();

        let temperature = match dict.get("Temperatura") {
            Some(val) => Some(parse_reading(val, "ºC", "temperature")?.parse::<f64>()?),
            None => None,
        };

        let humidity = match dict.get("Humedad") {
            Some(val) => Some(parse_reading(val, "%", "humidity")?.parse::<u64>()?),
            None => None,
        };

        let pressure = match dict.get("Presión") {
            Some(val) => Some(parse_reading(val, "hPa", "pressure")?.parse::<u64>()?),
            None => None,
        };

        let precipitation = match dict.get("Precip.") {
            Some(val) => Some(parse_reading(val, "mm", "precipitation")?.parse::<f64>()?),
            None => None,
        };

        let (wind_direction, wind_speed) = match dict.get("Viento") {
            Some(val) => {
                let re = Regex::new(r#"(.+)&nbsp;&nbsp;(.+)"#)?;
                let wind = re
                    .captures(val.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Failed to split wind"))?;
                let direction = wind
                    .get(1)
                    .ok_or_else(|| anyhow::anyhow!("Failed to extract wind direction"))?
                    .as_str()
                    .trim()
                    .replace("O", "W");
                let speed = wind
                    .get(2)
                    .ok_or_else(|| anyhow::anyhow!("Failed to extract wind speed"))?
                    .as_str()
                    .trim();

                let speed = parse_reading(speed, "km/h", "wind_speed")?.parse::<f64>()?;
                let speed_int = speed.round() as u64;

                (Some(direction), Some(speed_int))
            }
            None => (None, None),
        };

        let re = Regex::new(r#"Última actualización ?(\d\d-\d\d-\d\d\d\d \d\d:\d\d) ?UTC</td>"#)
            .unwrap();
        let delimiter = '-';
        let update_time = match re.captures(&body) {
            Some(captures) => match captures[1].split_once(' ') {
                Some((date, time)) => {
                    let date = date
                        .to_string()
                        .as_str()
                        .split(delimiter)
                        .rev()
                        .collect::<Vec<_>>()
                        .join(&delimiter.to_string());
                    Some(format!("{} {}", date, time))
                }
                None => None,
            },
            None => None,
        };

        Ok(Measurements {
            update_time,
            humidity,
            precipitation,
            pressure,
            temperature,
            wind_direction,
            wind_speed,
            gusts_speed: None,
        })
    }
}
