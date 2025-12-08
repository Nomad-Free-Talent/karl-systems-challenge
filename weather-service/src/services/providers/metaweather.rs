use serde::Deserialize;
use std::collections::HashMap;

const BASE_URL: &str = "https://www.metaweather.com/api";

#[derive(Debug, Clone)]
pub struct MetaWeatherProvider {
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct LocationSearchResponse {
    woeid: i64,
    title: String,
    #[serde(rename = "latt_long")]
    lat_long: String,
}

#[derive(Debug, Deserialize)]
struct WeatherData {
    id: i64,
    weather_state_name: String,
    weather_state_abbr: String,
    wind_direction_compass: String,
    created: String,
    applicable_date: String,
    min_temp: f64,
    max_temp: f64,
    the_temp: f64,
    wind_speed: f64,
    wind_direction: f64,
    air_pressure: f64,
    humidity: i64,
    visibility: f64,
    predictability: i64,
}

#[derive(Debug, Deserialize)]
struct ConsolidatedWeather {
    consolidated_weather: Vec<WeatherData>,
}

impl MetaWeatherProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn search_location(&self, city: &str) -> Result<Option<i64>, reqwest::Error> {
        let url = format!("{}/location/search/?query={}", BASE_URL, city);
        let response = self.client.get(&url).send().await?;
        
        if response.status().is_success() {
            let locations: Vec<LocationSearchResponse> = response.json().await?;
            Ok(locations.first().map(|l| l.woeid))
        } else {
            Ok(None)
        }
    }

    pub async fn get_weather(&self, woeid: i64) -> Result<serde_json::Value, reqwest::Error> {
        let url = format!("{}/location/{}/", BASE_URL, woeid);
        let response = self.client.get(&url).send().await?;
        
        let response = response.error_for_status()?;
        let data: ConsolidatedWeather = response.json().await?;
        
        // Extract current weather (first entry)
        if data.consolidated_weather.is_empty() {
            // Create a reqwest error by making a dummy request that will fail
            // This is a workaround since reqwest::Error doesn't have a simple constructor
            let dummy_url = "http://invalid-url-for-error-creation";
            let _ = self.client.get(dummy_url).send().await?;
            unreachable!();
        }
        
        let current = &data.consolidated_weather[0];
        
        let mut weather = HashMap::new();
        weather.insert("provider".to_string(), serde_json::json!("metaweather"));
        weather.insert("temperature".to_string(), serde_json::json!(current.the_temp));
        weather.insert("condition".to_string(), serde_json::json!(current.weather_state_name));
        weather.insert("humidity".to_string(), serde_json::json!(current.humidity));
        weather.insert("wind_speed".to_string(), serde_json::json!(current.wind_speed));
        weather.insert("wind_direction".to_string(), serde_json::json!(current.wind_direction_compass));
        weather.insert("pressure".to_string(), serde_json::json!(current.air_pressure));
        weather.insert("visibility".to_string(), serde_json::json!(current.visibility));
        
        Ok(serde_json::json!(weather))
    }

    pub async fn get_weather_for_city(&self, city: &str) -> Result<Option<serde_json::Value>, reqwest::Error> {
        if let Some(woeid) = self.search_location(city).await? {
            self.get_weather(woeid).await.map(Some)
        } else {
            Ok(None)
        }
    }
}

