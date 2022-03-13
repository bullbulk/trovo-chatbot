use std::collections::HashMap;
use std::error::Error;
use std::str;

use lazy_static::lazy_static;
use reqwest;
use reqwest::header::{HeaderMap, HeaderValue};
use portpicker::{pick_unused_port, Port};
use reqwest::{RequestBuilder, Response};

use crate::auth::structs::RefreshResponse;
use crate::auth::{db, server};
use crate::utils::config::{CONFIG, SCOPES};


lazy_static! {
    pub static ref HEADERS: HeaderMap = {
        let mut m = HeaderMap::new();
        m.insert("Accept", HeaderValue::from_str("application/json").unwrap());
        m.insert("Content-Type", HeaderValue::from_str("application/json").unwrap());
        m.insert("client-id", HeaderValue::from_str(&CONFIG.client_id.as_str()).unwrap());
        m
    };
}


pub async fn update_tokens(client: reqwest::Client) -> RefreshResponse {
    let token = db::read("refresh_token");

    let tokens = {
        if token.is_empty() {
            println!("Refresh token not found");
            run_oauth().await.unwrap()
        } else {
            println!("Refreshing tokens");
            let token = str::from_utf8(&*token).unwrap();
            refresh_tokens(client, token).await.unwrap()
        }
    };
    db::write("refresh_token", &tokens.refresh_token);

    return tokens;
}

pub async fn exchange_token(client: reqwest::Client, auth_code: &str, redirect_uri: String) -> Result<RefreshResponse, Box<dyn Error>> {
    let body = {
        let mut m = HashMap::new();
        m.insert("client_secret", CONFIG.client_secret.as_str());
        m.insert("grant_type", "authorization_code");
        m.insert("code", auth_code);
        m.insert("redirect_uri", redirect_uri.as_str());
        m
    };

    let request = client
        .post("https://open-api.trovo.live/openplatform/exchangetoken")
        .headers(HEADERS.to_owned())
        .json(&body);

    let response = request.send().await?;

    return match response.status() {
        reqwest::StatusCode::OK => {
            let payload = response.json::<RefreshResponse>().await?;
            Ok(payload)
        }
        _ => Err(format!("Caught an invalid response: {:?}", response))?
    };
}

async fn refresh_tokens(
    client: reqwest::Client, token: &str,
) -> Result<RefreshResponse, Box<dyn Error>> {
    let body: HashMap<&str, &str> = {
        let mut m: HashMap<&str, &str> = HashMap::new();
        m.insert("client_secret", CONFIG.client_secret.as_str());
        m.insert("grant_type", "refresh_token");
        m.insert("refresh_token", token);
        m
    };

    let request: RequestBuilder = client
        .post("https://open-api.trovo.live/openplatform/refreshtoken")
        .headers(HEADERS.to_owned())
        .json(&body);

    let response: Response = request.send().await?;

    return match response.status() {
        reqwest::StatusCode::OK => {
            let payload: RefreshResponse = response.json::<RefreshResponse>().await?;
            Ok(payload)
        }
        _ => Err(format!("Caught an invalid response: {:?}", response))?
    };
}

pub async fn run_oauth() -> Result<RefreshResponse, Box<dyn Error>> {
    let port: Port = pick_unused_port().unwrap();


    // User must open this link and login to account of bot
    let redirect_uri: String = format!("http://localhost:{}", port);
    let auth_url: String = format!(
        "Go to link:\nhttps://open.trovo.live/page/login.html?client_id={}&response_type=code&scope={}&redirect_uri={}",
        CONFIG.client_id, SCOPES.join("+"), redirect_uri
    );
    println!("{}", auth_url);

    // Out server is blocking the main thread and waiting for redirect from Trovo login page
    let code: String = server::oauth_server(port);
    // Get refresh and access token
    return exchange_token(reqwest::Client::new(), code.as_str(), redirect_uri).await;
}