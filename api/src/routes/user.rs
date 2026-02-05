use std::sync::{Arc, Mutex};

use poem::web::{Data, Json};
use poem::{
    handler
};

use crate::request_inputs::{CreateUserInput};
use crate::request_outputs::{CreateUserOutput};
use store::store::Store;

use crate::request_outputs::SignInOutput;
use crate::jwt;
use crate::password;

#[handler]
pub fn sign_up(Json(data): Json<CreateUserInput>, Data(s):Data<&Arc<Mutex<Store>>>) -> Result<Json<CreateUserOutput>, poem::Error> {

    let hashed_password = match password::hash_password(&data.password) {
        Ok(hash) => hash,
        Err(e) => {
            eprintln!("Password hashing error: {:?}", e);
            return Err(poem::Error::from_string(
                "Failed to process password",
                poem::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };
    let mut locked_s= s.lock().unwrap();
    match locked_s.sign_up(data.username.clone(), hashed_password) {
        Ok(id) => {
            let response = CreateUserOutput{
                id: id.to_string()
            };
            Ok(Json(response))
        }
        Err(e) => {
            eprintln!("Sign up error for user '{}': {:?}", data.username, e);
            Err(poem::Error::from_string(
                format!("Failed to create user: {:?}", e),
                poem::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

#[handler]
pub fn sign_in(Json(data): Json<CreateUserInput>, Data(s):Data<&Arc<Mutex<Store>>>) -> Result<Json<SignInOutput>, poem::Error> {
    let mut locked_s= s.lock().unwrap();
    match locked_s.sign_in(data.username.clone(), data.password.clone()) {
        Ok(user_id) => {
            match jwt::generate_jwt(&user_id) {
                Ok(token) => {
                    let response = SignInOutput {
                        jwt: token
                    };
                    Ok(Json(response))
                }
                Err(e) => {
                    eprintln!("JWT generation error: {:?}", e);
                    Err(poem::Error::from_string(
                        "Failed to generate token",
                        poem::http::StatusCode::INTERNAL_SERVER_ERROR,
                    ))
                }
            }
        }
        Err(e) => {
            eprintln!("Sign in error for user '{}': {:?}", data.username, e);
            Err(poem::Error::from_string(
                "Invalid username or password",
                poem::http::StatusCode::UNAUTHORIZED,
            ))
        }
    }
}