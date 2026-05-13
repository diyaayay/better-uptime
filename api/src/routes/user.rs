use crate::config::AppState;
use crate::jwt;
use crate::password;
use crate::request_inputs::CreateUserInput;
use crate::request_outputs::{CreateUserOutput, SignInOutput};
use diesel::result::{DatabaseErrorKind, Error as DieselError};
use poem::handler;
use poem::web::{Data, Json};
use std::sync::Arc;
use store::StoreError;

#[handler]
pub async fn sign_up(
    Json(data): Json<CreateUserInput>,
    Data(s): Data<&Arc<AppState>>,
) -> Result<Json<CreateUserOutput>, poem::Error> {
    validate_user_input(&data.username, &data.password)?;

    let hashed_password = password::hash_password(&data.password).map_err(|e| {
        eprintln!("Password hashing error: {:?}", e);
        poem::Error::from_string(
            "Failed to process password",
            poem::http::StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    match s.store.sign_up(data.username.clone(), hashed_password).await {
        Ok(id) => Ok(Json(CreateUserOutput { id })),
        Err(e) => {
            eprintln!("Sign up error for user '{}': {:?}", data.username, e);
            match e {
                StoreError::Diesel(DieselError::DatabaseError(
                    DatabaseErrorKind::UniqueViolation,
                    _,
                )) => Err(poem::Error::from_string(
                    "Username already exists",
                    poem::http::StatusCode::CONFLICT,
                )),
                _ => Err(poem::Error::from_string(
                    "Failed to create user",
                    poem::http::StatusCode::INTERNAL_SERVER_ERROR,
                )),
            }
        }
    }
}

#[handler]
pub async fn sign_in(
    Json(data): Json<CreateUserInput>,
    Data(s): Data<&Arc<AppState>>,
) -> Result<Json<SignInOutput>, poem::Error> {
    validate_user_input(&data.username, &data.password)?;

    let user_id = s
        .store
        .sign_in(data.username.clone(), data.password.clone())
        .await
        .map_err(|e| {
            eprintln!("Sign in error for user '{}': {:?}", data.username, e);
            poem::Error::from_string(
                "Invalid username or password",
                poem::http::StatusCode::UNAUTHORIZED,
            )
        })?;

    let token = jwt::generate_jwt(s.jwt_secret(), &user_id).map_err(|e| {
        eprintln!("JWT generation error: {:?}", e);
        poem::Error::from_string(
            "Failed to generate token",
            poem::http::StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    Ok(Json(SignInOutput { jwt: token }))
}

fn validate_user_input(username: &str, password: &str) -> Result<(), poem::Error> {
    if username.trim().is_empty() || password.trim().is_empty() {
        return Err(poem::Error::from_string(
        "Username or password must not be empty",
    poem::http::StatusCode::BAD_REQUEST));
    }

    if username.len() < 3 {
        return Err(poem::Error::from_string(
            "Username must be at least 3 characters",  // Fix: should say "Username"
            poem::http::StatusCode::BAD_REQUEST,
        ));
    }

    if password.len() < 8 {
        return Err(poem::Error::from_string(
            "Password must be at least 8 characters",
            poem::http::StatusCode::BAD_REQUEST,
        ));
    }
    Ok(())
}