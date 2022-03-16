use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;

use reqwest;
use reqwest::RequestBuilder;
use serde::de::DeserializeOwned;

use crate::api::chat::stream::ChatMessageStream;
use crate::api::errors::{EmptyError, InvalidResponse};
use crate::api::structs::{ChannelInfo, ChatTokenResponse, CommandResponse, DeleteResponse, MessageResponse, UserInfo, UsersResponse};
use crate::auth::auth::update_tokens;
use crate::utils::config::authorized_headers;

pub struct API {
    client: reqwest::Client,
    access_token: String,
}

impl API {
    // Need 'async' for awaiting 'update_tokens'
    pub async fn new() -> API {
        let _client = reqwest::Client::new();
        let tokens = update_tokens(_client).await;

        Self {
            client: reqwest::Client::new(),
            access_token: tokens.access_token,
        }
    }

    // In case of 401 status code, make 5 attempts with tokens refreshing, then return error
    async fn process_request<T: DeserializeOwned>(
        &mut self, request: RequestBuilder,
    ) -> Result<T, Box<dyn Error>> {

        // Empty error for 'possibly-uninitialized' satisfaction (E0381)
        let mut result: Result<T, Box<dyn Error>> = Err(EmptyError).map_err(|e| e.into());

        let mut attempt_counter = 0;

        for _ in 0..5 {
            // Replace 'Authorization' header with new access token
            let updated_request = request.try_clone().unwrap()
                .headers(
                    authorized_headers(self.access_token.clone())
                );
            let response = updated_request.send().await?;
            match response.status() {
                reqwest::StatusCode::OK => {
                    let payload = response.json::<T>().await?;
                    result = Ok(payload);
                    break;
                }
                // HTTP 401 (Incorrect access token)
                reqwest::StatusCode::UNAUTHORIZED => {
                    attempt_counter += 1;
                    if attempt_counter >= 5 {
                        result = Err(InvalidResponse {
                            code: response.status(),
                            response,
                        }).map_err(|e| e.into());
                        break;
                    }
                    // Refresh tokens
                    self.refresh().await;
                }
                // Any other code except 200 and 401
                _ => {
                    result = Err(InvalidResponse {
                        code: response.status(),
                        response,
                    }).map_err(|e| e.into());
                    break;
                }
            };
        };
        result
    }

    pub async fn refresh(&mut self) {
        let tokens = update_tokens(self.client.clone()).await;
        self.access_token = tokens.access_token;
    }

    pub async fn get_user_info(&mut self) -> Result<UserInfo, Box<dyn Error>> {
        let request = self.client
            .get("https://open-api.trovo.live/openplatform/getuserinfo");

        self.process_request::<UserInfo>(request).await
    }

    pub async fn get_users(
        &mut self, nicknames: Vec<String>,
    ) -> Result<UsersResponse, Box<dyn Error>> {
        let mut body = HashMap::new();
        body.insert("user", nicknames);

        let request = self.client
            .post("https://open-api.trovo.live/openplatform/getusers")
            .json(&body);

        self.process_request::<UsersResponse>(request).await
    }


    pub async fn get_channel_info(
        &mut self, channel_id: Option<i32>, username: Option<String>,
    ) -> Result<ChannelInfo, Box<dyn Error>> {
        let mut body = HashMap::new();
        if channel_id != None {
            body.insert("channel_id", channel_id.unwrap().to_string());
        }
        if username != None {
            body.insert("username", username.unwrap());
        }
        if body.is_empty() {
            panic!("No parameters provided");
        }

        let request = self.client
            .post("https://open-api.trovo.live/openplatform/channels/id")
            .json(&body);

        self.process_request::<ChannelInfo>(request).await
    }

    pub async fn send_my(
        &mut self, content: String,
    ) -> Result<MessageResponse, Box<dyn Error>> {
        let mut body = HashMap::new();
        body.insert("content", content);

        let request = self.client
            .post("https://open-api.trovo.live/openplatform/chat/send")
            .json(&body);

        self.process_request::<MessageResponse>(request).await
    }

    pub async fn send(
        &mut self, content: String, channel_id: i32,
    ) -> Result<MessageResponse, Box<dyn Error>> {
        let mut body = HashMap::new();
        body.insert("content", content);
        body.insert("channel_id", channel_id.to_string());

        let request = self.client
            .post("https://open-api.trovo.live/openplatform/chat/send")
            .json(&body);

        self.process_request::<MessageResponse>(request).await
    }

    // FIXME: Doesn't work at all. Server returns 400 HTTP and 20000 API status
    pub async fn delete(
        &mut self, channel_id: i32, message_id: String, sender_id: i32,
    ) -> Result<DeleteResponse, Box<dyn Error>> {
        let request = self.client
            .delete(
                format!(
                    "https://open-api.trovo.live/openplatform/channels/{}/messages/{}/users/{}",
                    channel_id.to_string(),
                    message_id,
                    sender_id.to_string()
                ));

        self.process_request::<DeleteResponse>(request).await
    }

    pub async fn chat_token(
        &mut self,
        channel_id: i32,
    ) -> Result<ChatTokenResponse, Box<dyn Error>> {
        let request = self.client
            .get(format!(
                "https://open-api.trovo.live/openplatform/chat/channel-token/{}",
                channel_id
            ));

        self.process_request::<ChatTokenResponse>(request).await
    }

    pub async fn chat_messages_for_channel(
        &mut self,
        channel_id: i32,
    ) -> Result<ChatMessageStream, Box<dyn Error>> {
        let token = self.chat_token(channel_id).await?;

        let messages = ChatMessageStream::connect(
            token.token.clone()
        ).await?;
        println!("Connected to chat");
        Ok(messages)
    }

    pub async fn command(
        &mut self, command: String, channel_id: i32,
    ) -> Result<CommandResponse, Box<dyn Error>> {
        let mut body = HashMap::new();
        body.insert("command", command);
        body.insert("channel_id", channel_id.to_string());

        let request = self.client
            .post("https://open-api.trovo.live/openplatform/channels/command")
            .json(&body);

        self.process_request::<CommandResponse>(request).await
    }

    // Display a list of moderator of this channel.
    pub async fn mods(
        &mut self, target_channel_id: i32,
    ) -> Result<CommandResponse, Box<dyn Error>> {
        let command = format!("mods");
        self.command(command, target_channel_id).await
    }

    // Display a list of banned users for this channel.
    pub async fn banned(
        &mut self, target_channel_id: i32,
    ) -> Result<CommandResponse, Box<dyn Error>> {
        let command = format!("banned");
        self.command(command, target_channel_id).await
    }

    // Duration is zero: Ban a user from chat permamently.
    // Duration is not zero: Ban a user from chat for 'duration'.
    pub async fn ban(
        &mut self, username: String, duration: Duration, target_channel_id: i32,
    ) -> Result<CommandResponse, Box<dyn Error>> {
        let command;
        if duration.is_zero() {
            command = format!("ban {}", username);
        } else {
            command = format!("ban {} {}s", username, duration.as_secs());
        }
        self.command(command, target_channel_id).await
    }

    // Remove ban on a user.
    pub async fn unban(
        &mut self, nickname: String, target_channel_id: i32,
    ) -> Result<CommandResponse, Box<dyn Error>> {
        let command = format!("unban {}", nickname);
        self.command(command, target_channel_id).await
    }

    // Grant moderator status to a user.
    pub async fn mod_(&mut self, nickname: String, target_channel_id: i32,
    ) -> Result<CommandResponse, Box<dyn Error>> {
        let command = format!("mod {}", nickname);
        self.command(command, target_channel_id).await
    }

    // Revoke moderator status from a user.
    pub async fn unmod(
        &mut self, nickname: String, target_channel_id: i32,
    ) -> Result<CommandResponse, Box<dyn Error>> {
        let command = format!("unmod {}", nickname);
        self.command(command, target_channel_id).await
    }

    // Clear chat history for all viewers.
    pub async fn clear(
        &mut self, target_channel_id: i32,
    ) -> Result<CommandResponse, Box<dyn Error>> {
        let command = format!("clear");
        self.command(command, target_channel_id).await
    }

    // Limit how frequently users can send messages in chat.
    pub async fn slow(
        &mut self, duration: Duration, target_channel_id: i32,
    ) -> Result<CommandResponse, Box<dyn Error>> {
        let command = format!("slow {}", duration.as_secs());
        self.command(command, target_channel_id).await
    }

    // Turn off slow mode.
    pub async fn slowoff(
        &mut self, target_channel_id: i32,
    ) -> Result<CommandResponse, Box<dyn Error>> {
        let command = format!("slowoff");
        self.command(command, target_channel_id).await
    }

    // Duration is zero: Restrict chat to followers based on their follow duration.
    // Duration is not zero: Restrict chat to followers only.
    pub async fn followers(
        &mut self, duration: Duration, target_channel_id: i32,
    ) -> Result<CommandResponse, Box<dyn Error>> {
        let command;
        if duration.is_zero() {
            command = format!("followers");
        } else {
            command = format!("followers {}s", duration.as_secs());
        }
        self.command(command, target_channel_id).await
    }

    // Turn off followers-only mode.
    pub async fn followersoff(
        &mut self, target_channel_id: i32,
    ) -> Result<CommandResponse, Box<dyn Error>> {
        let command = format!("followersoff");
        self.command(command, target_channel_id).await
    }

    // Stop live and hosting other channels.
    pub async fn host(
        &mut self, username: String, target_channel_id: i32,
    ) -> Result<CommandResponse, Box<dyn Error>> {
        let command = format!("host {}", username);
        self.command(command, target_channel_id).await
    }

    // Stop hosting channels.
    pub async fn unhost(
        &mut self, target_channel_id: i32,
    ) -> Result<CommandResponse, Box<dyn Error>> {
        let command = format!("unhost");
        self.command(command, target_channel_id).await
    }

    // Set title of your channel.
    pub async fn settitle(
        &mut self, title: String, target_channel_id: i32,
    ) -> Result<CommandResponse, Box<dyn Error>> {
        let command = format!("settitle {}", title);
        self.command(command, target_channel_id).await
    }

    // Set category of your channel.
    pub async fn setcategory(
        &mut self, category_name: String, target_channel_id: i32,
    ) -> Result<CommandResponse, Box<dyn Error>> {
        let command = format!("setcategory {}", category_name);
        self.command(command, target_channel_id).await
    }

    // Grant to user a custom role.
    pub async fn addrole(
        &mut self, rolename: String, username: String, target_channel_id: i32,
    ) -> Result<CommandResponse, Box<dyn Error>> {
        let command = format!("addrole {} {}", rolename, username);
        self.command(command, target_channel_id).await
    }

    // Revoke from user a custom role.
    pub async fn removerole(
        &mut self, rolename: String, username: String, target_channel_id: i32,
    ) -> Result<CommandResponse, Box<dyn Error>> {
        let command = format!("removerole {} {}", rolename, username);
        self.command(command, target_channel_id).await
    }

    // Fast clip the past 90-seconds stream in one channel.
    pub async fn fastclip(
        &mut self, target_channel_id: i32,
    ) -> Result<CommandResponse, Box<dyn Error>> {
        let command = format!("fastclip");
        self.command(command, target_channel_id).await
    }
}