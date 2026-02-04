use std::sync::{Arc, Mutex};

use poem::{ handler,web::{Data, Json, Path}
};

use crate::request_inputs::{CreateWebsiteInput};
use crate::request_outputs::{CreateWebsiteOutput, GetWebsiteOutput};
use store::store::Store;



#[handler]
pub fn get_website(Path(id): Path<String>, Data(s):Data<&Arc<Mutex<Store>>>) -> Json<GetWebsiteOutput> {
    let mut locked_s = s.lock().unwrap();
    let website = locked_s.get_website(id).unwrap();
    Json(GetWebsiteOutput{
        url: website.url
    })
}
#[handler]
pub fn create_website(Json(data):Json<CreateWebsiteInput>, Data(s):Data<&Arc<Mutex<Store>>>) -> Json<CreateWebsiteOutput> {
    let mut locked_s=s.lock().unwrap();
    let website = locked_s.create_website(String::from("ead27667-1b32-47e8-9252-38d26b47922b"), data.url).unwrap();

    let response = CreateWebsiteOutput{
        id: website.id
    };
    Json(response)
}

