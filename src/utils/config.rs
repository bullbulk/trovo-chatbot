use std::{env};
use std::path::Path;
use std::collections::{BTreeMap};

use envfile::EnvFile;
use serde::{Deserialize};

use lazy_static::lazy_static;


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


// Global singleton
lazy_static! {
    pub static ref CONFIG: Config = {
        Config::load_env();
        Config::get_config()
    };
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub client_id: String,
    pub client_secret: String,
    pub target_channel_username: String,
}

impl Config {
    // Call once
    pub fn load_env() -> () {
        // .env in application root (next to executable)
        let env_path = Path::new(".env");

        // Load variables from file to environment
        let env_store: BTreeMap<String, String> = EnvFile::new(env_path).unwrap().store;
        for (key, value) in &env_store {
            env::set_var(key, value);
        };
    }

    pub fn get_config() -> Config {
        // Load necessary environment variables to struct
        return envy::from_env::<Config>().unwrap();
    }
}