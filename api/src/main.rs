use std::sync::{Arc, Mutex};

use poem::{
    EndpointExt, Route, Server, get, listener::TcpListener, post
};

use store::store::Store;

use crate::routes::{user::{sign_up, sign_in}, website::get_website};
pub mod request_inputs;
pub mod request_outputs;
pub mod routes;
pub mod jwt;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {

    dotenvy::dotenv().ok();
  
    let s = Arc::new(Mutex::new(Store::new().unwrap()));
    let app = Route::new()
        .at("/website/:website_id", get(get_website))
        .at("/website", post(get_website))
        .at("/sign-up", post(sign_up))
        .at("/sign-in", post(sign_in))
        .data(s);
    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .name("hello-world")
        .run(app)
        .await
}