use diesel::Connection;
use diesel::pg::PgConnection;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../store/migrations");

pub fn run_pending_migrations(database_url: &str) -> Result<(), RunMigrationsError> {
    let mut conn = PgConnection::establish(database_url).map_err(RunMigrationsError::Connect)?;
    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| RunMigrationsError::Migrate(e.to_string()))?;
    Ok(())
}

#[derive(Debug)]
pub enum RunMigrationsError {
    Connect(diesel::ConnectionError),
    Migrate(String),
}

impl std::fmt::Display for RunMigrationsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RunMigrationsError::Connect(e) => write!(f, "database connection failed: {e}"),
            RunMigrationsError::Migrate(msg) => write!(f, "database migration failed: {msg}"),
        }
    }
}

impl std::error::Error for RunMigrationsError {}
