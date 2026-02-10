use std::sync::{Arc, Mutex};
use diesel::result::Error as DieselError;
use poem::{ handler,web::{Data, Json, Path}
};

use crate::{monitor::check_website, request_inputs::{CreateWebsiteInput, UpdateWebsiteInput}};
use crate::request_outputs::{CreateWebsiteOutput, GetWebsiteOutput, ListWebsiteOutput, WebsiteItem};
use store::store::Store;
use crate::auth::AuthUser;

#[derive(serde::Serialize)]
pub struct CheckNowOutput {
    pub is_up: bool,
    pub response_time_ms: Option<i32>,
    pub status_code: Option<i32>,
    pub error_message: Option<String>,
}

#[handler]
pub fn get_website(Path(id): Path<String>,
AuthUser(user_id) : AuthUser,
Data(s):Data<&Arc<Mutex<Store>>>)
-> Result<Json<GetWebsiteOutput>, poem::Error> {
    let mut locked_s = s.lock().unwrap();
    let website = locked_s.get_website(id.clone())
    .map_err(|e| {
        eprintln!("Error fetching website {}: {:?}", id, e);
        match e {
            diesel::result::Error::NotFound => {
                poem::Error::from_string(
                    "website not found",
                    poem::http::StatusCode::NOT_FOUND,
                )
            }
            _ => {
                poem::Error::from_string(
                    "Failed to fetch website",
                    poem::http::StatusCode::INTERNAL_SERVER_ERROR,
                )
            }
        }
    })?;
    if website.user_id != user_id {
        return Err(poem::Error::from_string(
            "You don't have permission to access this website",
            poem::http::StatusCode::FORBIDDEN,
        ));
    }
    Ok(Json(GetWebsiteOutput {
        url: website.url,
    }))
}

#[handler]
pub fn create_website(Json(data):Json<CreateWebsiteInput>,
AuthUser(user_id) : AuthUser,
Data(s):Data<&Arc<Mutex<Store>>>)
 -> Result<Json<CreateWebsiteOutput>, poem::Error> {
    if data.url.trim().is_empty() {
        return Err(poem::Error::from_string(
            "URL cannot be empty",
            poem::http::StatusCode::BAD_REQUEST
        ));
    }
    if !data.url.starts_with("http://") && !data.url.starts_with("https://") {
        return Err(poem::Error::from_string(
            "URL must start with http:// or https://",
            poem::http::StatusCode::BAD_REQUEST
        ));
    }
    let mut locked_s=s.lock().unwrap();
    let website = locked_s.create_website(
        user_id,
        data.url
    ).map_err(|e| {
        eprintln!("Error creating website: {:?}", e);
        poem::Error::from_string(
            "Failed to create website",
            poem::http::StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    Ok(Json(CreateWebsiteOutput {
        id: website.id,
    }))
}

#[handler]
pub fn list_websites(
    AuthUser(user_id) : AuthUser,
    Data(s): Data<&Arc<Mutex<Store>>>
) -> Result<Json<ListWebsiteOutput>, poem::Error> {
    let mut locked_s = s.lock().unwrap();
    let websites = locked_s.list_websites(user_id).map_err(|e| {
        eprintln!("Error listing websites: {:?}", e);
        poem::Error::from_string(
            "Failed to list websites",
            poem::http::StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;
    let items = websites.into_iter()
    .map(|w| WebsiteItem {
        id: w.id,
        url: w.url,
        time_added: w.time_added.format("%Y-%m-%dT%H:%M:%S").to_string()
    }).collect();
    Ok(Json(ListWebsiteOutput {
        items,
    }))
}
 #[handler]
 pub fn update_website(
    Path(id): Path<String>,
    Json(data): Json<UpdateWebsiteInput>,
    AuthUser(user_id) : AuthUser,
    Data(s) : Data<&Arc<Mutex<Store>>>,
 ) -> Result <Json<CreateWebsiteOutput>, poem::Error> {
    if data.url.trim().is_empty() {
        return Err(poem::Error::from_string(
            "URL cannot be empty",
            poem::http::StatusCode::BAD_REQUEST,
        ));
    }
    if !data.url.starts_with("http://") && !data.url.starts_with("https://") {
        return Err(poem::Error::from_string(
            "URL must start with http:// or https://",
            poem::http::StatusCode::BAD_REQUEST,
        ));
    }
    let mut locked_s = s.lock().unwrap();
    let website = locked_s.update_website(id.clone(), user_id, data.url).map_err(|e| {
        eprintln!("Error updating website {}: {:?}", id, e);
        match e {
            DieselError::NotFound => poem::Error::from_string(
                "Website not found or you don't have permission to update it",
                poem::http::StatusCode::NOT_FOUND,
            ),
            _ => poem::Error::from_string(
                "Failed to update website",
                poem::http::StatusCode::INTERNAL_SERVER_ERROR,
            ),
        }
    })?;
    Ok(Json(CreateWebsiteOutput { id:website.id }))
 } 
 
 #[handler]
 pub fn delete_website(
    Path(id): Path<String>,
    AuthUser(user_id): AuthUser,
    Data(s): Data<&Arc<Mutex<Store>>>
 ) -> Result <Json<serde_json::Value>, poem::Error> {
    let mut locked_s = s.lock().unwrap();
    locked_s
.delete_website(id.clone(), user_id)
.map_err(|e| {
    eprintln!("Error deleting website: {}: {:?}", id, e);
    match e {
        DieselError::NotFound => poem::Error::from_string(
            "Website not found or you don't have permission to delete it",
            poem::http::StatusCode::NOT_FOUND,
        ),
        _ => poem::Error::from_string(
            "Failed to delete website",
            poem::http::StatusCode::INTERNAL_SERVER_ERROR,
        ),
    }
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
    Data(s): Data<&Arc<Mutex<Store>>>,
) -> Result<Json<CheckNowOutput>, poem::Error> {
    // 1) DB access + auth check in its own block
    let url = {
        let mut locked = s.lock().unwrap();

        let website = locked.get_website(id.clone())
            .map_err(|e| {
                eprintln!("Error fetching website {}: {:?}", id, e);
                match e {
                    DieselError::NotFound => poem::Error::from_string(
                        "Website not found",
                        poem::http::StatusCode::NOT_FOUND,
                    ),
                    _ => poem::Error::from_string(
                        "Failed to fetch website",
                        poem::http::StatusCode::INTERNAL_SERVER_ERROR,
                    ),
                }
            })?;

        if website.user_id != user_id {
            return Err(poem::Error::from_string(
                "you don't have permission to access this website",
                poem::http::StatusCode::FORBIDDEN,
            ));
        }

        website.url.clone()  
    }; 
    let result = check_website(&url).await;

    {
        let mut locked = s.lock().unwrap();

        locked.record_check(
            id.clone(),
            result.is_up,
            result.response_time_ms,
            result.status_code,
            result.error_message.clone(),
        ).map_err(|e| {
            eprintln!("Error recording check for website {}: {:?}", id, e);
            poem::Error::from_string(
                "Failed to record check history",
                poem::http::StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        locked.update_website_status(
            id.clone(),
            result.is_up,
            result.response_time_ms,
        ).map_err(|e| {
            eprintln!("Error updating website status {}: {:?}", id, e);
            poem::Error::from_string(
                "Failed to update website status",
                poem::http::StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;
    }

    Ok(Json(CheckNowOutput {
        is_up: result.is_up,
        response_time_ms: result.response_time_ms,
        status_code: result.status_code,
        error_message: result.error_message,
    }))
}