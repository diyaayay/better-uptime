use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user ID
    pub exp: usize,  // expiration time
}

pub fn generate_jwt(
    secret: &[u8],
    user_id: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = (std::time::SystemTime::now()
        + std::time::Duration::from_secs(60 * 60 * 24)) // 24 hours
        .duration_since(std::time::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration,
    };

    let header = Header::new(Algorithm::HS256);
    encode(
        &header,
        &claims,
        &EncodingKey::from_secret(secret),
    )
}

pub fn verify_jwt(
    secret: &[u8],
    token: &str,
) -> Result<Claims, jsonwebtoken::errors::Error> {
    let validation = Validation::new(Algorithm::HS256);
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret),
        &validation,
    )
    .map(|data| data.claims)
}
