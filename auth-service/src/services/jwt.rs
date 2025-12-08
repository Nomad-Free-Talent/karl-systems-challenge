use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT token claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// User ID (subject)
    pub sub: Uuid,
    /// Username
    pub username: String,
    /// User roles
    pub roles: Vec<String>,
    /// Expiration timestamp
    pub exp: i64,
    /// Issued at timestamp
    pub iat: i64,
}

impl Claims {
    /// Create new claims with expiration set to 24 hours from now
    pub fn new(user_id: Uuid, username: String, roles: Vec<String>) -> Self {
        let now = Utc::now();
        let exp = now + Duration::hours(24); // Token expires in 24 hours

        Self {
            sub: user_id,
            username,
            roles,
            exp: exp.timestamp(),
            iat: now.timestamp(),
        }
    }
}

/// Generate a JWT token from claims
pub fn generate_token(claims: &Claims, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let header = Header::default();
    let encoding_key = EncodingKey::from_secret(secret.as_ref());
    
    encode(&header, claims, &encoding_key)
}

/// Validate a JWT token and extract claims
pub fn validate_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let decoding_key = DecodingKey::from_secret(secret.as_ref());
    let validation = Validation::default();
    
    let token_data = decode::<Claims>(token, &decoding_key, &validation)?;
    Ok(token_data.claims)
}

