use std::sync::{Arc, Mutex};

use poem::{
    EndpointExt, Route, Server, get, listener::TcpListener, post,
};

use store::store::Store;

use crate::routes::{
    user::{sign_in, sign_up},
    website::{
        create_website,
        get_website,
        list_websites,
        update_website,
        delete_website,
        check_website_now,
        get_website_status,
        get_website_history},
};
pub mod request_inputs;
pub mod request_outputs;
pub mod routes;
pub mod jwt;
pub mod password;  
pub mod auth;
pub mod monitor;
pub mod worker;
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {

    dotenvy::dotenv().ok();
  
    let s = Arc::new(Mutex::new(Store::new().unwrap()));

    crate::worker::start_background_worker(s.clone(), 60);
    println!("[Server] Starting API server on http://0.0.0.0:3000");

    let app = Route::new()
    .at("/websites", get(list_websites))
    .at("/website/:website_id", get(get_website).put(update_website).delete(delete_website))
    .at("/website", post(create_website))
    .at("/website/:website_id/check", get(check_website_now))
    .at("/website/:website_id/status", get(get_website_status))      
    .at("/website/:website_id/history", get(get_website_history)) 
    .at("/sign-up", post(sign_up))
    .at("/sign-in", post(sign_in))
    .data(s);
    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .name("hello-world")
        .run(app)
        .await
}