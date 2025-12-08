use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedWeatherData {
    pub data: serde_json::Value,
    pub cached_at: Instant,
    pub ttl: Duration,
}

impl CachedWeatherData {
    pub fn new(data: serde_json::Value, ttl: Duration) -> Self {
        Self {
            data,
            cached_at: Instant::now(),
            ttl,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.cached_at.elapsed() > self.ttl
    }
}

pub struct WeatherCache {
    cache: Arc<DashMap<String, CachedWeatherData>>,
    default_ttl: Duration,
}

impl WeatherCache {
    pub fn new(default_ttl_seconds: u64) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            default_ttl: Duration::from_secs(default_ttl_seconds),
        }
    }

    pub fn get(&self, key: &str) -> Option<serde_json::Value> {
        self.cache.get(key).and_then(|entry| {
            if entry.is_expired() {
                self.cache.remove(key);
                None
            } else {
                Some(entry.data.clone())
            }
        })
    }

    pub fn set(&self, key: String, value: serde_json::Value) {
        self.set_with_ttl(key, value, self.default_ttl);
    }

    pub fn set_with_ttl(&self, key: String, value: serde_json::Value, ttl: Duration) {
        let cached_data = CachedWeatherData::new(value, ttl);
        self.cache.insert(key, cached_data);
    }

    pub fn remove(&self, key: &str) -> Option<CachedWeatherData> {
        self.cache.remove(key).map(|(_, v)| v)
    }

    pub fn clear(&self) {
        self.cache.clear();
    }

    pub fn cleanup_expired(&self) {
        self.cache.retain(|_, v| !v.is_expired());
    }
}

