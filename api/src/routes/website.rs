use std::sync::{Arc, Mutex};

use poem::{ handler,web::{Data, Json, Path}
};

use crate::request_inputs::{CreateWebsiteInput};
use crate::request_outputs::{CreateWebsiteOutput, GetWebsiteOutput};
use store::store::Store;
use crate::auth::AuthUser;

#[handler]
pub fn get_website(Path(id): Path<String>,
AuthUser(user_id) : AuthUser,
Data(s):Data<&Arc<Mutex<Store>>>)
-> Result<Json<GetWebsiteOutput>, poem::Error> {
    let mut locked_s = s.lock().unwrap();
    let website = locked_s.get_website(id).unwrap();
    Ok(Json(GetWebsiteOutput{
        url: website.url
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

