use std::sync::Arc;

use poem::http::StatusCode;
use poem::web::Data;
use poem::{IntoResponse, handler};

use crate::config::AppState;

/// Liveness: process is running. Use for orchestrator / load balancer "is the binary up?" checks.
#[handler]
pub fn healthz() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

/// Readiness: database pool can serve a query. Returns **503** if Postgres is unreachable.
#[handler]
pub async fn readyz(Data(state): Data<&Arc<AppState>>) -> poem::Result<impl IntoResponse> {
    state.store.ping_db().await.map_err(|e| {
        poem::Error::from_string(
            format!("database unavailable: {e}"),
            StatusCode::SERVICE_UNAVAILABLE,
        )
    })?;
    Ok((StatusCode::OK, "ok"))
}
