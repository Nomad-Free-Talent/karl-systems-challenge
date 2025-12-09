use serde::{Deserialize, Serialize};

const BASE_URL: &str = "http://worldtimeapi.org/api";

#[derive(Debug, Clone)]
pub struct WorldTimeClient {
    client: reqwest::Client,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimezoneResponse {
    pub timezone: String,
    pub datetime: String,
    pub utc_offset: String,
    pub unixtime: i64,
}

impl Default for WorldTimeClient {
    fn default() -> Self {
        Self::new()
    }
}

impl WorldTimeClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_timezone(&self, timezone: &str) -> Result<TimezoneResponse, reqwest::Error> {
        let url = format!("{BASE_URL}/timezone/{timezone}");
        let response = self.client.get(&url).send().await?;

        let data: TimezoneResponse = response.json().await?;
        Ok(data)
    }

    pub async fn get_time_for_city(
        &self,
        city: &str,
    ) -> Result<Option<TimezoneResponse>, reqwest::Error> {
        // Map common city names to timezones (simplified)
        let timezone = match city.to_lowercase().as_str() {
            "london" => "Europe/London",
            "new york" | "nyc" => "America/New_York",
            "los angeles" | "la" => "America/Los_Angeles",
            "tokyo" => "Asia/Tokyo",
            "shanghai" => "Asia/Shanghai",
            "paris" => "Europe/Paris",
            "berlin" => "Europe/Berlin",
            "sydney" => "Australia/Sydney",
            "chicago" => "America/Chicago",
            "toronto" => "America/Toronto",
            _ => return Ok(None),
        };

        self.get_timezone(timezone).await.map(Some)
    }
}
