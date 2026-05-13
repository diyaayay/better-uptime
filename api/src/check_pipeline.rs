//! Persist HTTP check results and fire optional status webhooks.

use std::sync::Arc;

use store::{Store, StoreError};

use crate::monitor::CheckResult;
use crate::webhook_notify::{detect_status_transition, send_status_webhook_if_configured};

pub async fn record_check_update_status_notify(
    store: &Arc<Store>,
    website_id: String,
    monitored_url: String,
    prev_is_up: Option<bool>,
    webhook_url: Option<String>,
    result: &CheckResult,
) -> Result<(), StoreError> {
    store
        .record_check(
            website_id.clone(),
            result.is_up,
            result.response_time_ms,
            result.status_code,
            result.error_message.clone(),
        )
        .await?;

    store
        .update_website_status(website_id.clone(), result.is_up, result.response_time_ms)
        .await?;

    if let Some(transition) = detect_status_transition(prev_is_up, result.is_up) {
        send_status_webhook_if_configured(
            webhook_url.as_deref(),
            transition,
            &website_id,
            &monitored_url,
            result,
        )
        .await;
    }

    Ok(())
}
