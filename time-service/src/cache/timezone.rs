use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimezoneData {
    pub timezone: String,
    pub datetime: String,
    pub utc_offset: String,
    pub unix_time: i64,
}

pub struct TimezoneCache {
    cache: Arc<RwLock<HashMap<String, TimezoneData>>>,
}

impl TimezoneCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn initialize(&self) -> Result<(), String> {
        // Pre-populate with major cities/timezones
        let major_timezones = vec![
            "Europe/London",
            "America/New_York",
            "America/Los_Angeles",
            "Asia/Tokyo",
            "Asia/Shanghai",
            "Europe/Paris",
            "Europe/Berlin",
            "Australia/Sydney",
            "America/Chicago",
            "America/Toronto",
        ];

        for tz in major_timezones {
            if let Ok(data) = self.fetch_timezone_data(tz).await {
                let mut cache = self.cache.write().await;
                cache.insert(tz.to_string(), data);
            }
        }

        Ok(())
    }

    pub async fn get(&self, timezone: &str) -> Option<TimezoneData> {
        let cache = self.cache.read().await;
        cache.get(timezone).cloned()
    }

    pub async fn set(&self, timezone: String, data: TimezoneData) {
        let mut cache = self.cache.write().await;
        cache.insert(timezone, data);
    }

    pub async fn refresh(&self, timezone: &str) -> Result<(), String> {
        let data = self.fetch_timezone_data(timezone).await?;
        self.set(timezone.to_string(), data).await;
        Ok(())
    }

    async fn fetch_timezone_data(&self, timezone: &str) -> Result<TimezoneData, String> {
        let url = format!("http://worldtimeapi.org/api/timezone/{}", timezone);
        let client = reqwest::Client::new();
        let response = client.get(&url).send().await
            .map_err(|e| format!("Failed to fetch timezone data: {}", e))?;

        if response.status().is_success() {
            let data: serde_json::Value = response.json().await
                .map_err(|e| format!("Failed to parse response: {}", e))?;

            Ok(TimezoneData {
                timezone: data["timezone"].as_str().unwrap_or(timezone).to_string(),
                datetime: data["datetime"].as_str().unwrap_or("").to_string(),
                utc_offset: data["utc_offset"].as_str().unwrap_or("").to_string(),
                unix_time: data["unixtime"].as_i64().unwrap_or(0),
            })
        } else {
            Err(format!("API returned status: {}", response.status()))
        }
    }

    pub async fn list_timezones(&self) -> Vec<String> {
        let cache = self.cache.read().await;
        cache.keys().cloned().collect()
    }
}

