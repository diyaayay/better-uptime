use poem::{
    EndpointExt, Route, Server, get, listener::TcpListener, post,
};

use crate::config::{AppConfig, AppState};

use crate::routes::{
    user::{sign_in, sign_up},
    website::{
        check_website_now, create_website, delete_website, get_website, get_website_history,
        get_website_status, list_websites, update_website,
    },
};

pub mod auth;
pub mod config;
pub mod db_migrate;
pub mod jwt;
pub mod monitor;
pub mod password;
pub mod request_inputs;
pub mod request_outputs;
pub mod routes;
pub mod worker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    let config = AppConfig::from_env().unwrap_or_else(|e| {
        eprintln!("configuration error:\n{}", e);
        std::process::exit(1);
    });

    crate::db_migrate::run_pending_migrations(&config.database_url).unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });

    let state = AppState::try_new(config).unwrap_or_else(|e| {
        eprintln!("failed to open database pool: {}", e);
        std::process::exit(1);
    });

    crate::worker::start_background_worker(state.store.clone(), 60);
    println!("[Server] Starting API server on http://0.0.0.0:3000");

    let app = Route::new()
        .at("/websites", get(list_websites))
        .at(
            "/website/:website_id",
            get(get_website).put(update_website).delete(delete_website),
        )
        .at("/website", post(create_website))
        .at("/website/:website_id/check", get(check_website_now))
        .at("/website/:website_id/status", get(get_website_status))
        .at("/website/:website_id/history", get(get_website_history))
        .at("/sign-up", post(sign_up))
        .at("/sign-in", post(sign_in))
        .data(state);

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .name("hello-world")
        .run(app)
        .await?;

    Ok(())
}
