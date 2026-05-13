use std::sync::Arc;

use poem::http::StatusCode;
use poem::{
    FromRequest, Request, RequestBody, Result,
    web::headers::{Authorization, HeaderMapExt, authorization::Bearer},
};

use crate::config::AppState;
use crate::jwt::verify_jwt;

#[derive(Debug, Clone)]
pub struct AuthUser(pub String);

impl<'a> FromRequest<'a> for AuthUser {
    async fn from_request(req: &'a Request, _body: &mut RequestBody) -> Result<Self> {
        let state = req.data::<Arc<AppState>>().ok_or_else(|| {
            poem::Error::from_string(
                "Server misconfiguration: application state missing",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        let auth_header = req
            .headers()
            .typed_get::<Authorization<Bearer>>()
            .ok_or_else(|| {
                poem::Error::from_string("Missing Authorization header", StatusCode::UNAUTHORIZED)
            })?;

        let token = auth_header.token();
        let claims = verify_jwt(state.jwt_secret(), token).map_err(|e| {
            eprintln!("JWT verification error: {:?}", e);
            poem::Error::from_string("Invalid or expired token", StatusCode::UNAUTHORIZED)
        })?;

        Ok(AuthUser(claims.sub))
    }
}
