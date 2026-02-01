use serde::{Serialize,Deserialize};


#[derive(Serialize,Deserialize)]
pub struct CreateWebsiteInput{
   pub url: String,

}