use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CreateWebsiteInput {
    pub url: String,
    #[serde(default)]
    pub webhook_url: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateUserInput {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateWebsiteInput {
    pub url: String,
    /// Omitted = leave unchanged, JSON `null` = clear webhook, string = set.
    #[serde(default)]
    pub webhook_url: Option<Option<String>>,
}
