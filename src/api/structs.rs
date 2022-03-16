use std::fmt::Display;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde::de::{self, Deserializer};

fn num_from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where T: FromStr,
          T::Err: Display,
          D: Deserializer<'de>
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
}

// Use as UsersResponse.users.get(0).unwrap().clone().channel_id
#[derive(Serialize, Deserialize, Debug)]
pub struct UsersResponse {
    pub users: Vec<User>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    #[serde(deserialize_with = "num_from_str")]
    pub user_id: i32,
    #[serde(deserialize_with = "num_from_str")]
    pub channel_id: i32,
    pub username: String,
    pub nickname: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SocialLink {
    #[serde(alias = "type")]
    pub type_: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    #[serde(deserialize_with = "num_from_str")]
    pub user_id: i32,
    pub user_name: String,
    pub nick_name: String,
    pub email: String,
    pub profile_pic: String,
    pub info: String,
    #[serde(deserialize_with = "num_from_str")]
    pub channel_id: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelInfo {
    is_live: bool,
    #[serde(deserialize_with = "num_from_str")]
    pub category_id: i32,
    pub category_name: String,
    pub live_title: String,
    pub audi_type: String,
    pub language_code: String,
    pub thumbnail: String,
    pub current_viewers: i32,
    pub followers: i32,
    pub streamer_info: String,
    pub profile_pic: String,
    pub channel_url: String,
    #[serde(deserialize_with = "num_from_str")]
    pub created_at: i64,
    pub subscriber_num: i32,
    pub username: String,
    pub social_links: Vec<SocialLink>,
    #[serde(deserialize_with = "num_from_str")]
    pub started_at: i64,
    #[serde(deserialize_with = "num_from_str")]
    pub ended_at: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageResponse {}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteResponse {}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommandResponse {
    pub is_success: bool,
    pub display_msg: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatTokenResponse {
    pub token: String,
}
