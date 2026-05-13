use std::sync::Arc;

use diesel::result::Error as DieselError;
use poem::{
    handler,
    web::{Data, Json, Path, Query},
};

use crate::auth::AuthUser;
use crate::monitor::check_website;
use crate::request_inputs::{CreateWebsiteInput, UpdateWebsiteInput};
use crate::request_outputs::{
    CheckHistoryItem, CreateWebsiteOutput, GetWebsiteOutput, ListWebsiteOutput, WebsiteHistoryOutput,
    WebsiteItem, WebsiteStatusOutput,
};
use crate::config::AppState;
use store::StoreError;

#[derive(serde::Serialize)]
pub struct CheckNowOutput {
    pub is_up: bool,
    pub response_time_ms: Option<i32>,
    pub status_code: Option<i32>,
    pub error_message: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Map a `StoreError` into a `poem::Error` with a sensible default mapping:
/// `Diesel::NotFound` -> 404 with `not_found_msg`, everything else -> 500 with `fallback_msg`.
fn map_store_err(e: StoreError, not_found_msg: &'static str, fallback_msg: &'static str) -> poem::Error {
    match e {
        StoreError::Diesel(DieselError::NotFound) => {
            poem::Error::from_string(not_found_msg, poem::http::StatusCode::NOT_FOUND)
        }
        _ => poem::Error::from_string(fallback_msg, poem::http::StatusCode::INTERNAL_SERVER_ERROR),
    }
}

fn validate_url(url: &str) -> Result<(), poem::Error> {
    if url.trim().is_empty() {
        return Err(poem::Error::from_string(
            "URL cannot be empty",
            poem::http::StatusCode::BAD_REQUEST,
        ));
    }
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(poem::Error::from_string(
            "URL must start with http:// or https://",
            poem::http::StatusCode::BAD_REQUEST,
        ));
    }
    Ok(())
}

#[handler]
pub async fn get_website(
    Path(id): Path<String>,
    AuthUser(user_id): AuthUser,
    Data(s): Data<&Arc<AppState>>,
) -> Result<Json<GetWebsiteOutput>, poem::Error> {
    let website = s.store.get_website(id.clone()).await.map_err(|e| {
        eprintln!("Error fetching website {}: {:?}", id, e);
        map_store_err(e, "website not found", "Failed to fetch website")
    })?;

    if website.user_id != user_id {
        return Err(poem::Error::from_string(
            "You don't have permission to access this website",
            poem::http::StatusCode::FORBIDDEN,
        ));
    }

    Ok(Json(GetWebsiteOutput { url: website.url }))
}

#[handler]
pub async fn create_website(
    Json(data): Json<CreateWebsiteInput>,
    AuthUser(user_id): AuthUser,
    Data(s): Data<&Arc<AppState>>,
) -> Result<Json<CreateWebsiteOutput>, poem::Error> {
    validate_url(&data.url)?;

    let website = s.store.create_website(user_id, data.url).await.map_err(|e| {
        eprintln!("Error creating website: {:?}", e);
        poem::Error::from_string(
            "Failed to create website",
            poem::http::StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    Ok(Json(CreateWebsiteOutput { id: website.id }))
}

#[handler]
pub async fn list_websites(
    AuthUser(user_id): AuthUser,
    Data(s): Data<&Arc<AppState>>,
) -> Result<Json<ListWebsiteOutput>, poem::Error> {
    let websites = s.store.list_websites(user_id).await.map_err(|e| {
        eprintln!("Error listing websites: {:?}", e);
        poem::Error::from_string(
            "Failed to list websites",
            poem::http::StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    let items = websites
        .into_iter()
        .map(|w| WebsiteItem {
            id: w.id,
            url: w.url,
            time_added: w.time_added.format("%Y-%m-%dT%H:%M:%S").to_string(),
        })
        .collect();

    Ok(Json(ListWebsiteOutput { items }))
}

#[handler]
pub async fn update_website(
    Path(id): Path<String>,
    Json(data): Json<UpdateWebsiteInput>,
    AuthUser(user_id): AuthUser,
    Data(s): Data<&Arc<AppState>>,
) -> Result<Json<CreateWebsiteOutput>, poem::Error> {
    validate_url(&data.url)?;

    let website = s.store
        .update_website(id.clone(), user_id, data.url)
        .await
        .map_err(|e| {
            eprintln!("Error updating website {}: {:?}", id, e);
            map_store_err(
                e,
                "Website not found or you don't have permission to update it",
                "Failed to update website",
            )
        })?;

    Ok(Json(CreateWebsiteOutput { id: website.id }))
}

#[handler]
pub async fn delete_website(
    Path(id): Path<String>,
    AuthUser(user_id): AuthUser,
    Data(s): Data<&Arc<AppState>>,
) -> Result<Json<serde_json::Value>, poem::Error> {
    s.store.delete_website(id.clone(), user_id).await.map_err(|e| {
        eprintln!("Error deleting website {}: {:?}", id, e);
        map_store_err(
            e,
            "Website not found or you don't have permission to delete it",
            "Failed to delete website",
        )
    })?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Website deleted successfully",
    })))
}

#[handler]
pub async fn check_website_now(
    Path(id): Path<String>,
    AuthUser(user_id): AuthUser,
    Data(s): Data<&Arc<AppState>>,
) -> Result<Json<CheckNowOutput>, poem::Error> {
    let website = s.store.get_website(id.clone()).await.map_err(|e| {
        eprintln!("Error fetching website {}: {:?}", id, e);
        map_store_err(e, "Website not found", "Failed to fetch website")
    })?;

    if website.user_id != user_id {
        return Err(poem::Error::from_string(
            "you don't have permission to access this website",
            poem::http::StatusCode::FORBIDDEN,
        ));
    }

    let result = check_website(&website.url).await;

    s.store.record_check(
        id.clone(),
        result.is_up,
        result.response_time_ms,
        result.status_code,
        result.error_message.clone(),
    )
    .await
    .map_err(|e| {
        eprintln!("Error recording check for website {}: {:?}", id, e);
        poem::Error::from_string(
            "Failed to record check history",
            poem::http::StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    s.store.update_website_status(id.clone(), result.is_up, result.response_time_ms)
        .await
        .map_err(|e| {
            eprintln!("Error updating website status {}: {:?}", id, e);
            poem::Error::from_string(
                "Failed to update website status",
                poem::http::StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

    Ok(Json(CheckNowOutput {
        is_up: result.is_up,
        response_time_ms: result.response_time_ms,
        status_code: result.status_code,
        error_message: result.error_message,
    }))
}

#[handler]
pub async fn get_website_status(
    Path(id): Path<String>,
    AuthUser(user_id): AuthUser,
    Data(s): Data<&Arc<AppState>>,
) -> Result<Json<WebsiteStatusOutput>, poem::Error> {
    let website = s.store.get_website(id.clone()).await.map_err(|e| {
        eprintln!("Error fetching website {} for status: {:?}", id, e);
        map_store_err(e, "Website not found", "Failed to fetch the website status")
    })?;

    if website.user_id != user_id {
        return Err(poem::Error::from_string(
            "You don't have permission to access the website",
            poem::http::StatusCode::FORBIDDEN,
        ));
    }

    let last_checked = website
        .last_checked
        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%S").to_string());
    let last_down_time = website
        .last_down_time
        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%S").to_string());

    Ok(Json(WebsiteStatusOutput {
        is_up: website.is_up,
        last_checked,
        last_down_time,
        response_time_ms: website.response_time_ms,
    }))
}

#[handler]
pub async fn get_website_history(
    Path(id): Path<String>,
    AuthUser(user_id): AuthUser,
    Query(query): Query<HistoryQuery>,
    Data(s): Data<&Arc<AppState>>,
) -> Result<Json<WebsiteHistoryOutput>, poem::Error> {
    let website = s.store.get_website(id.clone()).await.map_err(|e| {
        eprintln!("Error fetching website {} for history: {:?}", id, e);
        map_store_err(e, "Website not found", "Failed to fetch the website history")
    })?;

    if website.user_id != user_id {
        return Err(poem::Error::from_string(
            "You don't have permission to access the website",
            poem::http::StatusCode::FORBIDDEN,
        ));
    }

    let limit = query.limit.unwrap_or(50).clamp(1, 500);
    let offset = query.offset.unwrap_or(0).max(0);

    let history = s
        .store
        .get_website_history(id.clone(), limit, offset)
        .await
        .map_err(|e| {
            eprintln!("Error fetching website history {}: {:?}", id, e);
            poem::Error::from_string(
                "failed to fetch website history",
                poem::http::StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

    let items = history
        .into_iter()
        .map(|h| CheckHistoryItem {
            checked_at: h.checked_at.format("%Y-%m-%dT%H:%M:%S").to_string(),
            is_up: h.is_up,
            response_time_ms: h.response_time_ms,
            status_code: h.status_code,
            error_message: h.error_message,
        })
        .collect();

    Ok(Json(WebsiteHistoryOutput { items }))
}
