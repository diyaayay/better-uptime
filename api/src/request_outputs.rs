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

#[derive(Serialize, Deserialize)]
pub struct WebsiteStatusOutput {
    pub is_up: Option<bool>,
    pub last_checked: Option<String>,
    pub last_down_time: Option<String>,
    pub response_time_ms: Option<i32>
    }

    #[derive(Serialize, Deserialize)]
    pub struct CheckHistoryItem {
        pub checked_at: String,
        pub is_up: bool,
        pub response_time_ms: Option<i32>,
        pub status_code: Option<i32>,
        pub error_message: Option<String>
    }

    #[derive(Serialize, Deserialize)]
    pub struct WebsiteHistoryOutput {
        pub items: Vec<CheckHistoryItem>
    }

