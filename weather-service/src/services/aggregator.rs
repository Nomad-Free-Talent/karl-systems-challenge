use std::collections::HashMap;
use crate::services::providers::{MetaWeatherProvider, OpenMeteoProvider};
use crate::services::{RateLimiter, WeatherProvider};
use serde_json::Value;

pub struct WeatherAggregator {
    metaweather: MetaWeatherProvider,
    openmeteo: OpenMeteoProvider,
    rate_limiter: RateLimiter,
}

#[derive(Debug, Serialize)]
pub struct AggregatedWeather {
    pub city: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub aggregated: AggregatedData,
    pub sources: Vec<Value>,
}

#[derive(Debug, Serialize)]
pub struct AggregatedData {
    pub temperature: f64,
    pub condition: String,
    pub humidity: Option<i64>,
    pub wind_speed: f64,
}

impl WeatherAggregator {
    pub fn new(rate_limiter: RateLimiter) -> Self {
        Self {
            metaweather: MetaWeatherProvider::new(),
            openmeteo: OpenMeteoProvider::new(),
            rate_limiter,
        }
    }

    pub async fn aggregate_weather(&self, city: &str) -> Result<AggregatedWeather, String> {
        let mut sources = Vec::new();
        let mut temperatures = Vec::new();
        let mut conditions = Vec::new();
        let mut humidities = Vec::new();
        let mut wind_speeds = Vec::new();

        // Fetch from MetaWeather
        self.rate_limiter.wait_if_needed(&WeatherProvider::MetaWeather).await;
        if let Ok(Some(weather)) = self.metaweather.get_weather_for_city(city).await {
            sources.push(weather.clone());
            
            if let Some(temp) = weather.get("temperature").and_then(|v| v.as_f64()) {
                temperatures.push(temp);
            }
            if let Some(cond) = weather.get("condition").and_then(|v| v.as_str()) {
                conditions.push(cond.to_string());
            }
            if let Some(hum) = weather.get("humidity").and_then(|v| v.as_i64()) {
                humidities.push(hum);
            }
            if let Some(ws) = weather.get("wind_speed").and_then(|v| v.as_f64()) {
                wind_speeds.push(ws);
            }
        }

        // Fetch from OpenMeteo
        self.rate_limiter.wait_if_needed(&WeatherProvider::OpenMeteo).await;
        if let Ok(Some(weather)) = self.openmeteo.get_weather_for_city(city).await {
            sources.push(weather.clone());
            
            if let Some(temp) = weather.get("temperature").and_then(|v| v.as_f64()) {
                temperatures.push(temp);
            }
            if let Some(cond) = weather.get("condition").and_then(|v| v.as_str()) {
                conditions.push(cond.to_string());
            }
            if let Some(ws) = weather.get("wind_speed").and_then(|v| v.as_f64()) {
                wind_speeds.push(ws);
            }
        }

        if sources.is_empty() {
            return Err("No weather data available from any provider".to_string());
        }

        // Aggregate data
        let avg_temp = if !temperatures.is_empty() {
            temperatures.iter().sum::<f64>() / temperatures.len() as f64
        } else {
            0.0
        };

        let most_common_condition = if !conditions.is_empty() {
            let mut condition_counts: HashMap<String, usize> = HashMap::new();
            for cond in &conditions {
                *condition_counts.entry(cond.clone()).or_insert(0) += 1;
            }
            condition_counts
                .into_iter()
                .max_by_key(|(_, count)| *count)
                .map(|(cond, _)| cond)
                .unwrap_or_else(|| "Unknown".to_string())
        } else {
            "Unknown".to_string()
        };

        let avg_humidity = if !humidities.is_empty() {
            Some((humidities.iter().sum::<i64>() as f64 / humidities.len() as f64) as i64)
        } else {
            None
        };

        let avg_wind_speed = if !wind_speeds.is_empty() {
            wind_speeds.iter().sum::<f64>() / wind_speeds.len() as f64
        } else {
            0.0
        };

        Ok(AggregatedWeather {
            city: city.to_string(),
            timestamp: chrono::Utc::now(),
            aggregated: AggregatedData {
                temperature: avg_temp,
                condition: most_common_condition,
                humidity: avg_humidity,
                wind_speed: avg_wind_speed,
            },
            sources,
        })
    }
}

