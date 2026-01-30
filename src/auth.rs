use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use chrono::{Utc, Duration};
use crate::models::{Claims, SystemUser};
use crate::error::AppError;

const JWT_SECRET: &[u8] = b"hcp_secret_key_change_me";

pub fn create_token(user: &SystemUser) -> Result<String, AppError> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: user.username.clone(),
        role: user.role.clone(),
        exp: expiration as usize,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(JWT_SECRET))
        .map_err(|e| AppError::InternalError(format!("Token creation failed: {}", e)))
}

pub fn verify_token(token: &str) -> Result<Claims, AppError> {
    let validation = Validation::new(Algorithm::HS256);
    decode::<Claims>(token, &DecodingKey::from_secret(JWT_SECRET), &validation)
        .map(|data| data.claims)
        .map_err(|_| AppError::AuthError("Invalid token".to_string()))
}
