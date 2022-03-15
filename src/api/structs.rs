use std::{error, fmt};
use std::fmt::Display;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde::de::{self, Deserializer};
use serde_repr::*;

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
pub struct CommandResponse {
    pub is_success: bool,
    pub display_msg: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatTokenResponse {
    pub token: String,
}


#[derive(Debug)]
pub struct InvalidResponse {
    pub code: reqwest::StatusCode,
    pub response: reqwest::Response,
}

impl fmt::Display for InvalidResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Caught an invalid response")
    }
}

impl error::Error for InvalidResponse {}

#[derive(Debug)]
pub struct EmptyError;

impl fmt::Display for EmptyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Empty error")
    }
}

impl error::Error for EmptyError {}


// Error codes returned by the Trovo api
#[derive(Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(i16)]
pub enum ErrorStatus {
    // Internal service failed to fetch data. Please try again.
    InternalFetch = -1201,

    // Internal server error. Try send the request again. In most cases, it is caused by timeout of
    // an internal service.
    InternalTimeout = -1000,

    // Server received invalid parameters. Please check the params you requested.
    InvalidParameters = 1002,

    // Unknown or uncategorized internal server error. Please report to developer@trovo.live
    InternalUnknown = 1111,

    // Conflict. Please try again.
    Conflict = 1203,

    // The user does not exist.
    InvalidUser = 10505,

    // Authorization failed. Please double check your token or the auth status.
    AuthorizationFailed = 10703,

    // Authorization Code doesn't exist or has expired
    InvalidAuthCode1 = 10710,

    // To avoid spam, one user cannot send the same message in 30 sec to a channel, or send more
    // than 1 message in 1 sec across all platforms. Streamers, Mods and Admins does not have this
    // limit. Give your bot mod access then you will not get this limit.
    MessageSpam = 10908,

    // The category does not exist.
    InvalidCategory = 11000,

    // Content conflicts with Trovo moderation rule.
    Moderated1 = 11101,

    // Content conflicts with Trovo moderation rule.
    Moderated2 = 11103,

    // The user account has been blocked by Trovo. To unblock the user, please contact us at
    // customer@trovo.live
    AccountBlocked = 11400,

    // Error in the request header.
    InvalidHeader = 11701,

    // Please try again with a valid scope.
    InvalidScope = 11703,

    // Double the access token you passed in is valid or not.
    InvalidAccessToken = 11704,

    // API rate limit exceeded. (You may apply for rate limit increase by contacting Trovo staff)
    RateLimitExceeded = 11706,

    // No permission to send chats to this channel.
    MissingChatPermission = 11707,

    // Invalid shard value. (Please make sure total_shard > 0 and 0 <= current_shard < total_shard)
    InvalidShardValue = 11708,

    // No permission to get the sharding token. Get shard token API is currently open to trusted
    // developers only. You may email developer@trovo.live to get whitelisted.
    MissingShardTokenPermission = 11709,

    // Authorization Code doesn't exist or has expired
    InvalidAuthCode2 = 11710,

    // Authorization Code has been used
    UsedAuthCode = 11711,

    // Refresh token has expired
    RefreshTokenExpired = 11712,

    // Invalid refresh token.
    InvalidRefreshToken = 11713,

    // Access token has expired
    AccessTokenExpired = 11714,

    // Invalid grant type
    InvalidGrantType = 11715,

    // Invalid Redirect URI
    InvalidRedirectUri = 11716,

    // Invalid client secret
    InvalidClientSecret = 11717,

    // Access token num is greater than 50, you should wait for the old access token to expire
    // before you can refresh again.
    AccessTokenLimit = 11718,

    // Scope is not authorized by the user.
    UnauthorizedScope = 11730,

    // The user is banned from chatting in this channel. Please contact the streamer/mods to unban
    // the user.
    BannedInChannel = 12400,

    // Channel is currently in slow mode. Please follow the slow mode rule to chat.
    SlowMode = 12401,

    // The streamer has set the channel to be follower only chat. Please follow the channel to
    // chat.
    FollowerOnly = 12402,

    // The user does not have permission to send hyperlinks in this channel. The channel is in
    // block hyperlink mode. Please check the hyperlink mode rules.
    UnauthorizedHyperlink = 12905,

    // Your message was moderated due to conflicts with the channel's moderation settings.
    ModeratedMessage = 12906,

    // Unknown or uncategorized error. Please report to developer@trovo.live.
    Unknown = 20000,
}

