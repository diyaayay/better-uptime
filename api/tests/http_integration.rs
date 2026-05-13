//! HTTP integration tests against a real Postgres (same env as CI: `DATABASE_URL`, `JWT_SECRET`).

use std::time::Duration;

use api::config::{AppConfig, AppState};
use api::{app_router, db_migrate};
use poem::{
    Server,
    listener::{Acceptor, Listener, TcpListener},
};
use serde_json::json;

fn require_env() -> (String, Vec<u8>) {
    let db_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set (e.g. CI or local docker compose + export DATABASE_URL)");
    let jwt = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set for integration tests");
    assert!(
        jwt.len() >= 32,
        "JWT_SECRET must be at least 32 characters (matches production validation)"
    );
    (db_url, jwt.into_bytes())
}

async fn spawn_test_server() -> (reqwest::Client, String, tokio::task::JoinHandle<()>) {
    let workspace_env = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../.env");
    let _ = dotenvy::from_path(&workspace_env);
    let _ = dotenvy::dotenv();
    let (database_url, jwt_secret) = require_env();
    db_migrate::run_pending_migrations(&database_url).expect("migrations");

    let state = AppState::try_new(AppConfig {
        database_url,
        jwt_secret,
    })
    .expect("AppState");

    let listener = TcpListener::bind("127.0.0.1:0");
    let acceptor = listener.into_acceptor().await.expect("bind acceptor");
    let local_addr = acceptor.local_addr().remove(0);
    let socket = local_addr.as_socket_addr().expect("socket addr");
    let base = format!("http://{}", socket);

    let app = app_router(state);
    let server = Server::new_with_acceptor(acceptor).name("integration-test");
    let handle = tokio::spawn(async move {
        let _ = server.run(app).await;
    });

    tokio::time::sleep(Duration::from_millis(150)).await;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("reqwest client");

    (client, base, handle)
}

#[tokio::test]
async fn healthz_and_readyz() {
    let (client, base, handle) = spawn_test_server().await;

    let r = client
        .get(format!("{}/healthz", base))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), reqwest::StatusCode::OK);
    assert_eq!(r.text().await.unwrap(), "ok");

    let r = client.get(format!("{}/readyz", base)).send().await.unwrap();
    assert_eq!(r.status(), reqwest::StatusCode::OK);
    assert_eq!(r.text().await.unwrap(), "ok");

    handle.abort();
}

#[tokio::test]
async fn sign_up_sign_in_create_list_website() {
    let (client, base, handle) = spawn_test_server().await;

    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let username = format!("t_{suffix}");
    let password = "testpass123";

    let r = client
        .post(format!("{}/sign-up", base))
        .json(&json!({ "username": username, "password": password }))
        .send()
        .await
        .unwrap();
    assert!(
        r.status().is_success(),
        "sign-up: {}",
        r.text().await.unwrap_or_default()
    );

    let r = client
        .post(format!("{}/sign-in", base))
        .json(&json!({ "username": username, "password": password }))
        .send()
        .await
        .unwrap();
    assert!(r.status().is_success(), "sign-in failed");
    let body: serde_json::Value = r.json().await.unwrap();
    let token = body["jwt"].as_str().expect("jwt in body");

    let r = client
        .post(format!("{}/website", base))
        .bearer_auth(token)
        .json(&json!({ "url": "https://example.com" }))
        .send()
        .await
        .unwrap();
    assert!(
        r.status().is_success(),
        "create website: {}",
        r.text().await.unwrap_or_default()
    );
    let created: serde_json::Value = r.json().await.unwrap();
    let website_id = created["id"].as_str().expect("website id");

    let r = client
        .get(format!("{}/websites", base))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert!(r.status().is_success());
    let list: serde_json::Value = r.json().await.unwrap();
    let items = list["items"].as_array().expect("items");
    assert!(
        items.iter().any(|it| it["id"].as_str() == Some(website_id)),
        "listed websites should include the new id"
    );

    handle.abort();
}
