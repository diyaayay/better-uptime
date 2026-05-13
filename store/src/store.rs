use deadpool_diesel::postgres::{Manager, Pool, Runtime};
use deadpool_diesel::{InteractError, PoolError};
use diesel::result::Error as DieselError;
use diesel::{RunQueryDsl, sql_query};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("failed to build connection pool: {0}")]
    Build(#[from] deadpool_diesel::postgres::BuildError),
    #[error("failed to acquire connection from pool: {0}")]
    Pool(#[from] PoolError),

    #[error("blocking task failed: {0}")]
    Interact(String),

    #[error(transparent)]
    Diesel(#[from] DieselError),
}

impl From<InteractError> for StoreError {
    fn from(err: InteractError) -> Self {
        StoreError::Interact(err.to_string())
    }
}

#[derive(Clone)]
pub struct Store {
    pub pool: Pool,
}

impl Store {
    /// Open a connection pool to the given PostgreSQL URL.
    pub fn connect(database_url: impl AsRef<str>) -> Result<Self, StoreError> {
        let manager = Manager::new(database_url.as_ref().to_string(), Runtime::Tokio1);
        let pool = Pool::builder(manager).max_size(16).build()?;
        Ok(Self { pool })
    }

    /// Cheap query to verify the database accepts connections (readiness probes).
    pub async fn ping_db(&self) -> Result<(), StoreError> {
        let conn = self.pool.get().await?;
        conn.interact(|conn| sql_query("SELECT 1").execute(conn).map(|_| ()))
            .await??;
        Ok(())
    }
}
