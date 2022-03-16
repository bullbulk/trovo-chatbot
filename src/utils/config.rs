use config::Config;
use lazy_static::lazy_static;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;

lazy_static! {
    pub static ref SETTINGS: Settings = get_settings();

    pub static ref HEADERS: HeaderMap = {
        let mut m = HeaderMap::new();
        m.insert("Accept", HeaderValue::from_str("application/json").unwrap());
        m.insert("Content-Type", HeaderValue::from_str("application/json").unwrap());
        m.insert("client-id", HeaderValue::from_str(SETTINGS.client_id.as_str()).unwrap());
        m
    };
}


// All available scopes
pub const SCOPES: [&str; 7] = [
    "user_details_self",
    "channel_details_self",
    "channel_update_self",
    "channel_subscriptions",
    "chat_send_self",
    "send_to_my_channel",
    "manage_messages"
];


#[derive(Deserialize, Debug, Clone)]
pub struct Settings {
    pub client_id: String,
    pub client_secret: String,
    pub target_channel_name: String,
}

fn get_settings() -> Settings {
    Config::builder()
        .add_source(config::File::with_name("settings.json"))
        .build().unwrap()
        .try_deserialize::<Settings>().unwrap()
}

pub fn headers() -> HeaderMap {
    HEADERS.to_owned()
}

pub fn authorized_headers(access_token: String) -> HeaderMap {
    let mut m = headers();
    m.insert("Authorization", HeaderValue::from_str(
        format!("OAuth {}", access_token).as_str()
    ).unwrap());
    m
}
