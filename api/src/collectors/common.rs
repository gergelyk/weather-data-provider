use crate::measurements::Measurements;
pub trait Downloader {
    fn base_url(&self) -> String;
    async fn try_download(&self, url: &str) -> anyhow::Result<Measurements>;

    async fn download(&self, url: &str) -> Measurements {
        let payload = self.try_download(url).await;
        match payload {
            Ok(payload) => {
                log::info!("Downloaded: {}", url);
                payload
            }
            Err(ref e) => {
                log::error!("{} while downloading: {}", e, url);
                Measurements::default()
            }
        }
    }
}

pub fn wind_direction_name(degrees: f64) -> &'static str {
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