use std::sync::Arc;

use store::Store;

use crate::monitor::check_website;

pub async fn check_all_websites(store: Arc<Store>) {
    println!("[Worker] Starting check cycle for all websites...");

    let websites = match store.get_all_websites().await {
        Ok(websites) => websites,
        Err(e) => {
            eprintln!("[Worker] Error fetching websites: {:?}", e);
            return;
        }
    };

    if websites.is_empty() {
        println!("[Worker] No websites to check");
        return;
    }

    println!("[Worker] Checking {} websites...", websites.len());

    for website in websites {
        let website_id = website.id.clone();
        let url = website.url.clone();

        println!("[Worker] checking website {}: {}", website_id, url);

        let result = check_website(&url).await;

        if result.is_up {
            println!(
                "[Worker] ✓ {} is UP ({}ms, status {})",
                url,
                result.response_time_ms.unwrap_or(0),
                result.status_code.unwrap_or(0),
            );
        } else {
            println!(
                "[Worker] ✗ {} is DOWN: {}",
                url,
                result.error_message.as_deref().unwrap_or("Unknown error")
            );
        }

        match store
            .record_check(
                website_id.clone(),
                result.is_up,
                result.response_time_ms,
                result.status_code,
                result.error_message.clone(),
            )
            .await
        {
            Ok(_) => println!("[Worker] Recorded check history for {}", website_id),
            Err(e) => eprintln!("[Worker] Error recording check: {:?}", e),
        }

        match store
            .update_website_status(website_id.clone(), result.is_up, result.response_time_ms)
            .await
        {
            Ok(_) => println!("[Worker] Updated status for {}", website_id),
            Err(e) => eprintln!("[Worker] Error updating status: {:?}", e),
        }
    }

    println!("[Worker] Finished check cycle");
}

/// Background worker that periodically checks all websites.
pub fn start_background_worker(store: Arc<Store>, interval_seconds: u64) {
    tokio::spawn(async move {
        let mut interval =
            tokio::time::interval(tokio::time::Duration::from_secs(interval_seconds));
        interval.tick().await;

        println!(
            "[Worker] Background worker started (checking every {} seconds)",
            interval_seconds
        );

        loop {
            interval.tick().await;
            check_all_websites(store.clone()).await;
        }
    });
}
