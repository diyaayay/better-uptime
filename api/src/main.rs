use poem::{
    Route, Server, get, handler, listener::TcpListener, post, web::{Json, Path}
};

use request_inputs::{CreateWebsiteInput, CreateUserInput};
use request_outputs::{CreateWebsiteOutput, CreateUserOutput, GetWebsiteOutput};
use store::store::Store;

use crate::request_outputs::SignInOutput;
pub mod request_inputs;
pub mod request_outputs;

#[handler]
fn get_website(Path(id): Path<String>) -> Json<GetWebsiteOutput> {
    let mut s=Store::new().unwrap();
    let website = s.get_website(id).unwrap();
    Json(GetWebsiteOutput{
        url: website.url
    })
}
#[handler]
fn create_website(Json(data):Json<CreateWebsiteInput>) -> Json<CreateWebsiteOutput> {
    let mut s=Store::new().unwrap();
    let website = s.create_website(String::from("ead27667-1b32-47e8-9252-38d26b47922b"), data.url).unwrap();

    let response = CreateWebsiteOutput{
        id: website.id
    };
    Json(response)
}

#[handler]
fn sign_up(Json(data): Json<CreateUserInput>) -> Json<CreateUserOutput> {
    let mut s=Store::new().unwrap();
    let id = s.sign_up(data.username, data.password).unwrap();
    let response = CreateUserOutput{
        id: id.to_string()
    };
    Json(response)
}

#[handler]
fn sign_in(Json(data): Json<CreateUserInput>) -> Json<SignInOutput> {
    let mut s=Store::new().unwrap();
    // let exists = s.sign_in(data.username, data.password).unwrap();
    let response = SignInOutput{
        jwt: String::from("Diya")
    };
    Json(response)
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {

    let app = Route::new()
        .at("/website/:website_id", get(get_website))
        .at("/website", post(create_website))
        .at("/sign-up", post(sign_up))
        .at("/sign-in", post(sign_in));
    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .name("hello-world")
        .run(app)
        .await
}