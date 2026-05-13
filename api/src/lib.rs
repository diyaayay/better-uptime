//! Library crate for the HTTP API (binary in `main.rs` links here; integration tests use `api::`).

use std::sync::Arc;

use poem::{EndpointExt, IntoEndpoint, Route, get, post};

pub mod auth;
pub mod config;
pub mod db_migrate;
pub mod health;
pub mod jwt;
pub mod monitor;
pub mod password;
pub mod request_inputs;
pub mod request_outputs;
pub mod routes;
pub mod worker;

use crate::config::AppState;
use crate::health::{healthz, readyz};
use crate::routes::user::{sign_in, sign_up};
use crate::routes::website::{
    check_website_now, create_website, delete_website, get_website, get_website_history,
    get_website_status, list_websites, update_website,
};

/// Same route tree as production; does not start the background worker.
pub fn app_router(state: Arc<AppState>) -> impl IntoEndpoint {
    Route::new()
        .at("/healthz", get(healthz))
        .at("/readyz", get(readyz))
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
        .data(state)
}
