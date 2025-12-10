use crate::services::providers::{OpenMeteoProvider, WttrInProvider};
use crate::services::{RateLimiter, WeatherProvider};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

/// Aggregates weather data from multiple providers
pub struct WeatherAggregator {
    wttrin: WttrInProvider,
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
            wttrin: WttrInProvider::new(),
            openmeteo: OpenMeteoProvider::new(),
            rate_limiter,
        }
    }

    /// Aggregate weather data from all available providers for a given city
    /// Returns aggregated temperature, condition, humidity, and wind speed
    pub async fn aggregate_weather(&self, city: &str) -> Result<AggregatedWeather, String> {
        // Check rate limits for both providers concurrently
        let (_, _) = tokio::join!(
            self.rate_limiter.wait_if_needed(&WeatherProvider::WttrIn),
            self.rate_limiter
                .wait_if_needed(&WeatherProvider::OpenMeteo)
        );

        // Fetch from both providers concurrently with timeout handling
        const API_TIMEOUT_SECS: u64 = 10;
        let timeout_duration = std::time::Duration::from_secs(API_TIMEOUT_SECS);

        let (wttrin_result, openmeteo_result) = tokio::join!(
            tokio::time::timeout(timeout_duration, self.wttrin.get_weather_for_city(city)),
            tokio::time::timeout(timeout_duration, self.openmeteo.get_weather_for_city(city))
        );

        let mut sources = Vec::new();
        let mut temperatures = Vec::new();
        let mut conditions = Vec::new();
        let mut humidities = Vec::new();
        let mut wind_speeds = Vec::new();

        // Process wttr.in result with error handling
        match wttrin_result {
            Ok(Ok(Some(weather))) => {
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
            Ok(Ok(None)) => {
                log::debug!("wttr.in returned no data for city: {}", city);
            }
            Ok(Err(e)) => {
                log::warn!("wttr.in API error for city {}: {}", city, e);
            }
            Err(_) => {
                log::warn!(
                    "wttr.in API timeout for city: {} (exceeded {}s)",
                    city,
                    API_TIMEOUT_SECS
                );
            }
        }

        // Process OpenMeteo result with error handling
        match openmeteo_result {
            Ok(Ok(Some(weather))) => {
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
            Ok(Ok(None)) => {
                log::debug!("OpenMeteo returned no data for city: {}", city);
            }
            Ok(Err(e)) => {
                log::warn!("OpenMeteo API error for city {}: {}", city, e);
            }
            Err(_) => {
                log::warn!(
                    "OpenMeteo API timeout for city: {} (exceeded {}s)",
                    city,
                    API_TIMEOUT_SECS
                );
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
