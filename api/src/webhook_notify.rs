//! Outbound status webhooks (POST JSON) when a monitored site transitions up/down.

use std::time::Duration;

use reqwest::Client;
use serde::Serialize;
use tracing::{error, warn};

use crate::monitor::CheckResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusTransition {
    ToDown,
    ToUp,
}

/// Detect a meaningful transition for alerting (ignore unknown → down on first checks).
pub fn detect_status_transition(
    prev_is_up: Option<bool>,
    now_up: bool,
) -> Option<StatusTransition> {
    match (prev_is_up, now_up) {
        (Some(true), false) => Some(StatusTransition::ToDown),
        (Some(false), true) => Some(StatusTransition::ToUp),
        _ => None,
    }
}

#[derive(Serialize)]
struct WebhookPayload<'a> {
    event: &'a str,
    website_id: &'a str,
    url: &'a str,
    is_up: bool,
    response_time_ms: Option<i32>,
    status_code: Option<i32>,
    error_message: Option<&'a str>,
}

pub async fn send_status_webhook_if_configured(
    webhook_url: Option<&str>,
    transition: StatusTransition,
    website_id: &str,
    monitored_url: &str,
    result: &CheckResult,
) {
    let Some(url) = webhook_url.map(str::trim).filter(|u| !u.is_empty()) else {
        return;
    };

    let event = match transition {
        StatusTransition::ToDown => "website.down",
        StatusTransition::ToUp => "website.up",
    };

    let client = match Client::builder()
        .timeout(Duration::from_secs(8))
        .user_agent("better-uptime-webhook/1")
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            error!(error = %e, "webhook client build failed");
            return;
        }
    };

    let body = WebhookPayload {
        event,
        website_id,
        url: monitored_url,
        is_up: result.is_up,
        response_time_ms: result.response_time_ms,
        status_code: result.status_code,
        error_message: result.error_message.as_deref(),
    };

    match client.post(url).json(&body).send().await {
        Ok(resp) if resp.status().is_success() => {
            tracing::info!(%url, %website_id, %event, "webhook delivered");
        }
        Ok(resp) => {
            warn!(
                %url,
                %website_id,
                %event,
                status = %resp.status(),
                "webhook returned non-success status"
            );
        }
        Err(e) => {
            error!(%url, %website_id, %event, error = %e, "webhook request failed");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{StatusTransition, detect_status_transition};

    #[test]
    fn detects_down_from_up() {
        assert_eq!(
            detect_status_transition(Some(true), false),
            Some(StatusTransition::ToDown)
        );
    }

    #[test]
    fn detects_up_from_down() {
        assert_eq!(
            detect_status_transition(Some(false), true),
            Some(StatusTransition::ToUp)
        );
    }

    #[test]
    fn ignores_unknown_previous() {
        assert_eq!(detect_status_transition(None, false), None);
        assert_eq!(detect_status_transition(None, true), None);
    }

    #[test]
    fn ignores_no_state_change() {
        assert_eq!(detect_status_transition(Some(true), true), None);
        assert_eq!(detect_status_transition(Some(false), false), None);
    }
}
