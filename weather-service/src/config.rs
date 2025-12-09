use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    #[allow(dead_code)] // Reserved for future service-to-service communication
    pub auth_service_url: String,
    pub jwt_secret: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        let auth_service_url =
            env::var("AUTH_SERVICE_URL").unwrap_or_else(|_| "http://localhost:8000".to_string());

        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

        let port = env::var("PORT")
            .unwrap_or_else(|_| "8001".to_string())
            .parse::<u16>()
            .expect("PORT must be a valid number");

        Self {
            auth_service_url,
            jwt_secret,
            port,
        }
    }
}
