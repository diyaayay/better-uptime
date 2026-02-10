use std::time::Duration;
use reqwest::Client;

pub struct CheckResult {
    pub is_up: bool,
    pub response_time_ms: Option<i32>,
    pub status_code: Option<i32>,
    pub error_message: Option<String>,
}

pub async fn check_website(url: &str) -> CheckResult {
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();

    let start = std::time::Instant::now();

    match client.get(url).send().await {
        Ok(resp) => {
            let elapsed_ms = start.elapsed().as_millis() as i32;
            let status = resp.status();
            CheckResult {
                is_up: status.is_success(),
                response_time_ms: Some(elapsed_ms),
                status_code: Some(status.as_u16() as i32),
                error_message: if status.is_success() {
                    None
                } else {
                    Some(format!("HTTP {}", status.as_u16()))
                },
            }
        }
        Err(e) => {
            let elapsed_ms = start.elapsed().as_millis() as i32;
            CheckResult {
                is_up: false,
                response_time_ms: Some(elapsed_ms),
                status_code: None,
                error_message: Some(e.to_string()),
            }
        }
    }
}
