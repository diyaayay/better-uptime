use std::env;
use std::sync::Arc;

use store::{Store, StoreError};

/// Minimum length for `JWT_SECRET` when using HS256.
/// Shorter secrets are easier to brute-force; 32 bytes is a common baseline.
const MIN_JWT_SECRET_LEN: usize = 32;

#[derive(Debug)]
pub struct AppConfig {
    pub database_url: String,
    pub jwt_secret: Vec<u8>,
}

impl AppConfig {
    /// Call after [`dotenvy::dotenv`] in `main` if you use a `.env` file.
    pub fn from_env() -> Result<Self, String> {
        let database_url = env::var("DATABASE_URL").map_err(|_| {
            "DATABASE_URL must be set (example: postgresql://user:pass@127.0.0.1:5432/betteruptime)"
                .to_string()
        })?;

        if database_url.trim().is_empty() {
            return Err("DATABASE_URL must not be empty".to_string());
        }

        let jwt_raw = env::var("JWT_SECRET").map_err(|_| {
            "JWT_SECRET must be set — generate one with: openssl rand -base64 32".to_string()
        })?;

        if jwt_raw.len() < MIN_JWT_SECRET_LEN {
            return Err(format!(
                "JWT_SECRET must be at least {} characters (got {}). Use a longer secret, e.g. `openssl rand -base64 32`",
                MIN_JWT_SECRET_LEN,
                jwt_raw.len()
            ));
        }

        Ok(Self {
            database_url,
            jwt_secret: jwt_raw.into_bytes(),
        })
    }
}

/// Handlers receive `Data<&Arc<AppState>>` and use [`AppState::store`] for the database
/// and [`AppState::jwt_secret`] for signing and verification (see `auth` and `sign_in`).
pub struct AppState {
    pub store: Arc<Store>,
    jwt_secret: Vec<u8>,
}

impl AppState {
    pub fn try_new(config: AppConfig) -> Result<Arc<Self>, StoreError> {
        let store = Arc::new(Store::connect(&config.database_url)?);
        Ok(Arc::new(Self {
            store,
            jwt_secret: config.jwt_secret,
        }))
    }

    /// Bytes used as the HMAC key for HS256 JWTs.
    pub fn jwt_secret(&self) -> &[u8] {
        &self.jwt_secret
    }
}
