use crate::cache::timezone::TimezoneData;
use crate::cache::TimezoneCache;
use crate::services::WorldTimeClient;
use actix_web::{web, HttpResponse, Responder};
use serde::Serialize;
use shared::{ApiResponse, AppError, AppResult};

#[derive(Debug, Serialize)]
pub struct TimeResponse {
    pub city: String,
    pub timezone: String,
    pub datetime: String,
    pub utc_offset: String,
    pub unix_time: i64,
}

pub async fn get_time_for_city(
    cache: web::Data<TimezoneCache>,
    client: web::Data<WorldTimeClient>,
    path: web::Path<String>,
) -> AppResult<impl Responder> {
    let city = path.into_inner();

    // Map city to timezone
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
        _ => return Err(AppError::NotFound(format!("City not supported: {}", city))),
    };

    // Check cache first (this is the primary source)
    if let Some(cached_data) = cache.get(timezone).await {
        let response = TimeResponse {
            city: city.clone(),
            timezone: cached_data.timezone,
            datetime: cached_data.datetime,
            utc_offset: cached_data.utc_offset,
            unix_time: cached_data.unix_time,
        };
        return Ok(HttpResponse::Ok().json(ApiResponse::new(response)));
    }

    // If not in cache, try to fetch from API (may fail, but worth trying)
    // This will only succeed if API is available
    // Use a timeout to prevent hanging on slow/failed API calls
    match tokio::time::timeout(
        std::time::Duration::from_secs(5),
        client.get_time_for_city(&city)
    ).await {
        Ok(Ok(Some(api_data))) => {
            // Cache the result for future use
            cache
                .set(
                    timezone.to_string(),
                    TimezoneData {
                        timezone: api_data.timezone.clone(),
                        datetime: api_data.datetime.clone(),
                        utc_offset: api_data.utc_offset.clone(),
                        unix_time: api_data.unixtime,
                    },
                )
                .await;

            let response = TimeResponse {
                city: city.clone(),
                timezone: api_data.timezone,
                datetime: api_data.datetime,
                utc_offset: api_data.utc_offset,
                unix_time: api_data.unixtime,
            };
            return Ok(HttpResponse::Ok().json(ApiResponse::new(response)));
        }
        Ok(Ok(None)) => {
            // City not supported by API
            return Err(AppError::NotFound(format!(
                "City not supported: {}",
                city
            )));
        }
        Ok(Err(e)) => {
            // API call failed - log but don't expose internal error
            log::warn!("API call failed for city {}: {}", city, e);
        }
        Err(_) => {
            // Timeout - API is too slow
            log::warn!("API call timed out for city: {}", city);
        }
    }

    // If both cache and API fail, return error
    Err(AppError::NotFound(format!(
        "Time data not found for city: {} (cache miss and API unavailable). Please try again later.",
        city
    )))
}

pub async fn get_time_for_timezone(
    cache: web::Data<TimezoneCache>,
    client: web::Data<WorldTimeClient>,
    path: web::Path<String>,
) -> AppResult<impl Responder> {
    let timezone = path.into_inner();

    // Check cache first (primary source)
    if let Some(cached) = cache.get(&timezone).await {
        let response = TimeResponse {
            city: timezone.clone(),
            timezone: cached.timezone,
            datetime: cached.datetime,
            utc_offset: cached.utc_offset,
            unix_time: cached.unix_time,
        };
        return Ok(HttpResponse::Ok().json(ApiResponse::new(response)));
    }

    // If not in cache, try to fetch from API (may fail, but worth trying)
    // Use a timeout to prevent hanging on slow/failed API calls
    match tokio::time::timeout(
        std::time::Duration::from_secs(5),
        client.get_timezone(&timezone)
    ).await {
        Ok(Ok(timezone_data)) => {
            // Cache the result for future use
            cache
                .set(
                    timezone.clone(),
                    TimezoneData {
                        timezone: timezone_data.timezone.clone(),
                        datetime: timezone_data.datetime.clone(),
                        utc_offset: timezone_data.utc_offset.clone(),
                        unix_time: timezone_data.unixtime,
                    },
                )
                .await;

            let response = TimeResponse {
                city: timezone.clone(),
                timezone: timezone_data.timezone,
                datetime: timezone_data.datetime,
                utc_offset: timezone_data.utc_offset,
                unix_time: timezone_data.unixtime,
            };
            return Ok(HttpResponse::Ok().json(ApiResponse::new(response)));
        }
        Ok(Err(e)) => {
            // API call failed - log but don't expose internal error
            log::warn!("API call failed for timezone {}: {}", timezone, e);
        }
        Err(_) => {
            // Timeout - API is too slow
            log::warn!("API call timed out for timezone: {}", timezone);
        }
    }

    // If both cache and API fail, return error
    Err(AppError::NotFound(format!(
        "Time data not found for timezone: {} (cache miss and API unavailable). Please try again later.",
        timezone
    )))
}

pub async fn list_timezones(cache: web::Data<TimezoneCache>) -> AppResult<impl Responder> {
    let timezones = cache.list_timezones().await;
    Ok(HttpResponse::Ok().json(ApiResponse::new(timezones)))
}
