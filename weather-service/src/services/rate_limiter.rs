use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub enum WeatherProvider {
    MetaWeather,
    OpenMeteo,
}

impl WeatherProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            WeatherProvider::MetaWeather => "metaweather",
            WeatherProvider::OpenMeteo => "openmeteo",
        }
    }
}

pub struct RateLimiter {
    last_calls: Arc<Mutex<HashMap<String, Instant>>>,
    min_delay: Duration,
}

impl RateLimiter {
    pub fn new(min_delay_seconds: u64) -> Self {
        Self {
            last_calls: Arc::new(Mutex::new(HashMap::new())),
            min_delay: Duration::from_secs(min_delay_seconds),
        }
    }

    pub async fn wait_if_needed(&self, provider: &WeatherProvider) {
        let provider_name = provider.as_str();
        let mut last_calls = self.last_calls.lock().await;

        if let Some(last_call) = last_calls.get(provider_name) {
            let elapsed = last_call.elapsed();
            if elapsed < self.min_delay {
                let wait_time = self.min_delay - elapsed;
                drop(last_calls); // Release lock before sleeping
                tokio::time::sleep(wait_time).await;
            }
        }

        // Update last call time
        last_calls.insert(provider_name.to_string(), Instant::now());
    }

    pub async fn can_make_request(&self, provider: &WeatherProvider) -> bool {
        let provider_name = provider.as_str();
        let last_calls = self.last_calls.lock().await;

        if let Some(last_call) = last_calls.get(provider_name) {
            last_call.elapsed() >= self.min_delay
        } else {
            true
        }
    }
}

