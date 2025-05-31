use serde_json::json;

#[derive(serde::Serialize, Default, Debug)]
pub struct Measurements {
    pub update_time: Option<String>,
    pub humidity: Option<u64>,
    pub precipitation: Option<f64>,
    pub pressure: Option<u64>,
    pub temperature: Option<f64>,
    pub wind_direction: Option<String>,
    pub wind_speed: Option<u64>,
    pub gusts_speed: Option<u64>,
}

pub fn get_units() -> serde_json::Value {
    let units = json!({
        "update_time": "UTC",
        "humidity": "%",
        "precipitation": "mm",
        "pressure": "hPa",
        "temperature": "\u{00B0}C",
        "wind_direction": "",
        "wind_speed": "km/h",
        "gusts_speed": "km/h",
    });
    units
}
