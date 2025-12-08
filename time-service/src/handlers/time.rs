use actix_web::{web, HttpResponse, Responder};
use serde::Serialize;
use shared::{ApiResponse, AppError, AppResult};
use crate::cache::TimezoneCache;
use crate::services::WorldTimeClient;

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

    // Try to get from cache first
    if let Some(cached) = client.get_time_for_city(&city).await
        .map_err(|e| AppError::Internal(format!("Failed to fetch time: {}", e)))?
    {
        let response = TimeResponse {
            city: city.clone(),
            timezone: cached.timezone,
            datetime: cached.datetime,
            utc_offset: cached.utc_offset,
            unix_time: cached.unixtime,
        };
        return Ok(HttpResponse::Ok().json(ApiResponse::new(response)));
    }

    // If not in cache, fetch from API
    let timezone_data = cache.get(&city).await;
    
    if let Some(data) = timezone_data {
        let response = TimeResponse {
            city: city.clone(),
            timezone: data.timezone,
            datetime: data.datetime,
            utc_offset: data.utc_offset,
            unix_time: data.unix_time,
        };
        Ok(HttpResponse::Ok().json(ApiResponse::new(response)))
    } else {
        Err(AppError::NotFound(format!("Time data not found for city: {}", city)))
    }
}

pub async fn get_time_for_timezone(
    cache: web::Data<TimezoneCache>,
    client: web::Data<WorldTimeClient>,
    path: web::Path<String>,
) -> AppResult<impl Responder> {
    let timezone = path.into_inner();

    // Check cache first
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

    // Fetch from API
    let timezone_data = client.get_timezone(&timezone).await
        .map_err(|e| AppError::Internal(format!("Failed to fetch timezone: {}", e)))?;

    // Cache it
    cache.set(timezone.clone(), crate::cache::TimezoneData {
        timezone: timezone_data.timezone.clone(),
        datetime: timezone_data.datetime.clone(),
        utc_offset: timezone_data.utc_offset.clone(),
        unix_time: timezone_data.unixtime,
    }).await;

    let response = TimeResponse {
        city: timezone.clone(),
        timezone: timezone_data.timezone,
        datetime: timezone_data.datetime,
        utc_offset: timezone_data.utc_offset,
        unix_time: timezone_data.unixtime,
    };

    Ok(HttpResponse::Ok().json(ApiResponse::new(response)))
}

pub async fn list_timezones(
    cache: web::Data<TimezoneCache>,
) -> AppResult<impl Responder> {
    let timezones = cache.list_timezones().await;
    Ok(HttpResponse::Ok().json(ApiResponse::new(timezones)))
}

