use std::sync::Arc;

use store::Store;
use tracing::{error, info};

use crate::check_pipeline::record_check_update_status_notify;
use crate::monitor::check_website;

pub async fn check_all_websites(store: Arc<Store>) {
    info!("worker check cycle: fetching websites");

    let websites = match store.get_all_websites().await {
        Ok(websites) => websites,
        Err(e) => {
            error!(error = ?e, "worker failed to list websites");
            return;
        }
    };

    if websites.is_empty() {
        info!("worker check cycle: no websites");
        return;
    }

    info!(
        count = websites.len(),
        "worker check cycle: checking websites"
    );

    for website in websites {
        let website_id = website.id.clone();
        let monitored_url = website.url.clone();
        let prev_is_up = website.is_up;
        let webhook_url = website.webhook_url.clone();

        info!(%website_id, %monitored_url, "worker: checking");

        let result = check_website(&monitored_url).await;

        if result.is_up {
            info!(
                %monitored_url,
                ms = ?result.response_time_ms,
                status = ?result.status_code,
                "worker: site up",
            );
        } else {
            info!(
                %monitored_url,
                err = %result.error_message.as_deref().unwrap_or("unknown"),
                "worker: site down",
            );
        }

        match record_check_update_status_notify(
            &store,
            website_id.clone(),
            monitored_url.clone(),
            prev_is_up,
            webhook_url,
            &result,
        )
        .await
        {
            Ok(()) => info!(%website_id, "worker: persisted check and status"),
            Err(e) => error!(%website_id, error = ?e, "worker: failed to persist check/status"),
        }
    }

    info!("worker check cycle: finished");
}

/// Background worker that periodically checks all websites.
pub fn start_background_worker(store: Arc<Store>, interval_seconds: u64) {
    tokio::spawn(async move {
        let mut interval =
            tokio::time::interval(tokio::time::Duration::from_secs(interval_seconds));
        interval.tick().await;

        info!(
            interval_secs = interval_seconds,
            "background worker started",
        );

        loop {
            interval.tick().await;
            check_all_websites(store.clone()).await;
        }
    });
}
