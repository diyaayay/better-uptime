use serde::{Serialize,Deserialize};


#[derive(Serialize,Deserialize)]
pub struct CreateWebsiteOutput {
    pub id: String
}

#[derive(Serialize,Deserialize)]
pub struct CreateUserOutput {
    pub id: String
}

#[derive(Serialize,Deserialize)]
pub struct SignInOutput {
    pub jwt: String
}

#[derive(Serialize,Deserialize)]
pub struct GetWebsiteOutput {
    pub url: String
}

#[derive(Serialize, Deserialize)]
pub struct ListWebsiteOutput {
    pub items: Vec<WebsiteItem>
}

#[derive(Serialize, Deserialize)]
pub struct WebsiteItem {
    pub id: String,
    pub url: String,
    pub time_added: String
}
