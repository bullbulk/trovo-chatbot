use reqwest;

use trovo_chatbot::auth::auth;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let tokens = auth::update_tokens(client).await;
    println!("{:?}", tokens);
    Ok(())
}
