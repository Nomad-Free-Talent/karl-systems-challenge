use crate::cache::WeatherCache;
use crate::services::WeatherAggregator;
use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use shared::{ApiResponse, AppError, AppResult};

pub async fn get_weather(
    cache: web::Data<WeatherCache>,
    aggregator: web::Data<WeatherAggregator>,
    path: web::Path<String>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> AppResult<impl Responder> {
    let city = path.into_inner();
    let force_refresh = query.get("cache").map(|v| v == "false").unwrap_or(false);

    // Check cache first (unless force refresh)
    if !force_refresh {
        if let Some(cached_data) = cache.get(&city) {
            return Ok(HttpResponse::Ok().json(ApiResponse::new(cached_data)));
        }
    }

    // Fetch fresh data
    let aggregated = aggregator
        .aggregate_weather(&city)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to aggregate weather: {e}")))?;

    let response_data = json!({
        "city": aggregated.city,
        "timestamp": aggregated.timestamp,
        "aggregated": {
            "temperature": aggregated.aggregated.temperature,
            "condition": aggregated.aggregated.condition,
            "humidity": aggregated.aggregated.humidity,
            "wind_speed": aggregated.aggregated.wind_speed,
        },
        "sources": aggregated.sources,
    });

    // Cache the result
    cache.set(city.clone(), response_data.clone());

    Ok(HttpResponse::Ok().json(ApiResponse::new(response_data)))
}

pub async fn get_weather_providers(
    aggregator: web::Data<WeatherAggregator>,
    path: web::Path<String>,
) -> AppResult<impl Responder> {
    let city = path.into_inner();

    let aggregated = aggregator
        .aggregate_weather(&city)
        .await
        .map_err(|e| AppError::Internal(format!("Failed to aggregate weather: {e}")))?;

    Ok(HttpResponse::Ok().json(ApiResponse::new(aggregated.sources)))
}
