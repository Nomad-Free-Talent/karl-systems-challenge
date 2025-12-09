use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use shared::Claims as SharedClaims;
use uuid::Uuid;

/// Re-export shared Claims for convenience
pub use shared::Claims;

/// Helper function to create Claims with expiration set to 24 hours from now
pub fn create_claims(
    user_id: Uuid,
    username: String,
    roles: Vec<String>,
    permissions: Vec<String>,
) -> SharedClaims {
    let now = Utc::now();
    let exp = now + Duration::hours(24); // Token expires in 24 hours

    SharedClaims {
        sub: user_id,
        username,
        roles,
        permissions,
        exp: exp.timestamp(),
        iat: now.timestamp(),
    }
}

/// Generate a JWT token from claims
pub fn generate_token(
    claims: &SharedClaims,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let header = Header::default();
    let encoding_key = EncodingKey::from_secret(secret.as_ref());

    encode(&header, claims, &encoding_key)
}

/// Validate a JWT token and extract claims
pub fn validate_token(
    token: &str,
    secret: &str,
) -> Result<SharedClaims, jsonwebtoken::errors::Error> {
    let decoding_key = DecodingKey::from_secret(secret.as_ref());
    let validation = Validation::default();

    let token_data = decode::<SharedClaims>(token, &decoding_key, &validation)?;
    Ok(token_data.claims)
}
