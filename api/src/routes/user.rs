use std::sync::{Arc, Mutex};

use poem::web::{Data, Json};
use poem::{
    handler
};

use crate::request_inputs::{CreateUserInput};
use crate::request_outputs::{CreateUserOutput};
use store::store::Store;

use crate::request_outputs::SignInOutput;

#[handler]
pub fn sign_up(Json(data): Json<CreateUserInput>, Data(s):Data<&Arc<Mutex<Store>>>) -> Json<CreateUserOutput> {
    let mut locked_s= s.lock().unwrap();
    let id = locked_s.sign_up(data.username, data.password).unwrap();
    let response = CreateUserOutput{
        id: id.to_string()
    };
    Json(response)
}

#[handler]
pub fn sign_in(Json(data): Json<CreateUserInput>, Data(s):Data<&Arc<Mutex<Store>>>) -> Json<SignInOutput> {
    let mut locked_s= s.lock().unwrap();
    let _exists = locked_s.sign_in(data.username, data.password).unwrap();
    let response = SignInOutput{
        jwt: String::from("Diya")
    };
    Json(response)
}