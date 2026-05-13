use poem::{Server, listener::TcpListener};

use api::config::{AppConfig, AppState};
use api::{app_router, db_migrate, worker};

fn init_tracing() {
    use tracing_subscriber::EnvFilter;
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,poem=info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    init_tracing();

    let config = AppConfig::from_env().unwrap_or_else(|e| {
        eprintln!("configuration error:\n{}", e);
        std::process::exit(1);
    });

    db_migrate::run_pending_migrations(&config.database_url).unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });

    let state = AppState::try_new(config).unwrap_or_else(|e| {
        eprintln!("failed to open database pool: {}", e);
        std::process::exit(1);
    });

    worker::start_background_worker(state.store.clone(), 60);
    tracing::info!("listening on http://0.0.0.0:3000");

    let app = app_router(state);

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .name("hello-world")
        .run(app)
        .await?;

    Ok(())
}
