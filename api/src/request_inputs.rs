use serde::{Serialize,Deserialize};


#[derive(Serialize,Deserialize)]
pub struct CreateWebsiteInput{
   pub url: String,

}

#[derive(Serialize,Deserialize)]
pub struct CreateUserInput {
   pub username: String,
   pub password: String
}

#[derive(Serialize,Deserialize)]
pub struct UpdateWebsiteInput {
   pub url:String
}
