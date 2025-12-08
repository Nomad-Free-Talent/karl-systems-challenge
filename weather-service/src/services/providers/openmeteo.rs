use serde::Deserialize;
use std::collections::HashMap;

const BASE_URL: &str = "https://api.open-meteo.com/v1";

#[derive(Debug, Clone)]
pub struct OpenMeteoProvider {
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Fields needed for deserialization but not all are used
struct GeocodingResponse {
    results: Vec<GeocodingResult>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Fields needed for deserialization but not all are used
struct GeocodingResult {
    latitude: f64,
    longitude: f64,
    name: String,
    country: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Fields needed for deserialization but not all are used
struct CurrentWeather {
    temperature: f64,
    windspeed: f64,
    winddirection: f64,
    weathercode: i64,
    time: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Fields needed for deserialization but not all are used
struct WeatherResponse {
    current: CurrentWeather,
    current_units: HashMap<String, String>,
}

impl OpenMeteoProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn geocode_city(&self, city: &str) -> Result<Option<(f64, f64)>, reqwest::Error> {
        let url = format!("{}/search?name={}", BASE_URL.replace("/v1", ""), city);
        let response = self.client.get(&url).send().await?;
        
        if response.status().is_success() {
            let data: GeocodingResponse = response.json().await?;
            if let Some(result) = data.results.first() {
                Ok(Some((result.latitude, result.longitude)))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub async fn get_weather(&self, latitude: f64, longitude: f64) -> Result<serde_json::Value, reqwest::Error> {
        let url = format!(
            "{}/forecast?latitude={}&longitude={}&current_weather=true",
            BASE_URL, latitude, longitude
        );
        let response = self.client.get(&url).send().await?;
        let response = response.error_for_status()?;
        let data: WeatherResponse = response.json().await?;
        
        // Map weather code to condition (simplified)
        let condition = match data.current.weathercode {
            0 => "Clear sky",
            1..=3 => "Partly cloudy",
            45..=48 => "Foggy",
            51..=67 => "Rainy",
            71..=77 => "Snowy",
            80..=86 => "Rainy",
            95..=99 => "Thunderstorm",
            _ => "Unknown",
        };
        
        let mut weather = HashMap::new();
        weather.insert("provider".to_string(), serde_json::json!("openmeteo"));
        weather.insert("temperature".to_string(), serde_json::json!(data.current.temperature));
        weather.insert("condition".to_string(), serde_json::json!(condition));
        weather.insert("wind_speed".to_string(), serde_json::json!(data.current.windspeed));
        weather.insert("wind_direction".to_string(), serde_json::json!(data.current.winddirection));
        
        Ok(serde_json::json!(weather))
    }

    pub async fn get_weather_for_city(&self, city: &str) -> Result<Option<serde_json::Value>, reqwest::Error> {
        if let Some((lat, lon)) = self.geocode_city(city).await? {
            self.get_weather(lat, lon).await.map(Some)
        } else {
            Ok(None)
        }
    }
}

