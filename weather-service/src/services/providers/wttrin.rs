use serde::Deserialize;
use std::collections::HashMap;

const BASE_URL: &str = "https://wttr.in";

#[derive(Debug, Clone)]
pub struct WttrInProvider {
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Fields needed for deserialization but not all are used
struct CurrentCondition {
    #[serde(rename = "FeelsLikeC")]
    feels_like_c: String,
    #[serde(rename = "FeelsLikeF")]
    feels_like_f: String,
    #[serde(rename = "cloudcover")]
    cloud_cover: String,
    #[serde(rename = "humidity")]
    humidity: String,
    #[serde(rename = "localObsDateTime")]
    local_obs_date_time: String,
    #[serde(rename = "observation_time")]
    observation_time: String,
    #[serde(rename = "precipMM")]
    precip_mm: String,
    #[serde(rename = "pressure")]
    pressure: String,
    #[serde(rename = "temp_C")]
    temp_c: String,
    #[serde(rename = "temp_F")]
    temp_f: String,
    #[serde(rename = "uvIndex")]
    uv_index: String,
    #[serde(rename = "visibility")]
    visibility: String,
    #[serde(rename = "weatherCode")]
    weather_code: String,
    #[serde(rename = "weatherDesc")]
    weather_desc: Vec<WeatherDesc>,
    #[serde(rename = "weatherIconUrl")]
    weather_icon_url: Vec<WeatherIconUrl>,
    #[serde(rename = "winddir16Point")]
    wind_dir_16_point: String,
    #[serde(rename = "winddirDegree")]
    wind_dir_degree: String,
    #[serde(rename = "windspeedKmph")]
    wind_speed_kmph: String,
    #[serde(rename = "windspeedMiles")]
    wind_speed_miles: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct WeatherDesc {
    value: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct WeatherIconUrl {
    value: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)] // Fields needed for deserialization but not all are used
struct WttrInResponse {
    current_condition: Vec<CurrentCondition>,
}

impl Default for WttrInProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl WttrInProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("Mozilla/5.0") // wttr.in requires a user agent
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    pub async fn get_weather_for_city(
        &self,
        city: &str,
    ) -> Result<Option<serde_json::Value>, reqwest::Error> {
        // wttr.in API format: https://wttr.in/{city}?format=j1
        let url = format!("{BASE_URL}/{city}?format=j1");
        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let data: WttrInResponse = response.json().await?;

        if data.current_condition.is_empty() {
            return Ok(None);
        }

        let current = &data.current_condition[0];

        // Parse temperature (comes as string)
        let temperature = current.temp_c.parse::<f64>().unwrap_or(0.0);
        let humidity = current.humidity.parse::<i64>().unwrap_or(0);
        let wind_speed_kmph = current.wind_speed_kmph.parse::<f64>().unwrap_or(0.0);
        // Convert km/h to m/s (approximate conversion, or keep as km/h)
        // wttr.in provides km/h, but we'll convert to m/s to match other providers
        let wind_speed_ms = wind_speed_kmph / 3.6;

        let condition = current
            .weather_desc
            .first()
            .map(|d| d.value.clone())
            .unwrap_or_else(|| "Unknown".to_string());

        let mut weather = HashMap::new();
        weather.insert("provider".to_string(), serde_json::json!("wttrin"));
        weather.insert("temperature".to_string(), serde_json::json!(temperature));
        weather.insert("condition".to_string(), serde_json::json!(condition));
        weather.insert("humidity".to_string(), serde_json::json!(humidity));
        weather.insert("wind_speed".to_string(), serde_json::json!(wind_speed_ms));
        weather.insert(
            "wind_direction".to_string(),
            serde_json::json!(current.wind_dir_16_point),
        );
        weather.insert(
            "pressure".to_string(),
            serde_json::json!(current.pressure.parse::<f64>().unwrap_or(0.0)),
        );
        weather.insert(
            "visibility".to_string(),
            serde_json::json!(current.visibility.parse::<f64>().unwrap_or(0.0)),
        );

        Ok(Some(serde_json::json!(weather)))
    }
}
