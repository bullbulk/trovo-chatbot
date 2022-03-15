use std::collections::HashMap;
use std::error::Error;

use reqwest;
use reqwest::RequestBuilder;
use serde::de::DeserializeOwned;

use crate::api::stream::stream::ChatMessageStream;
use crate::api::structs::{ChannelInfo, ChatTokenResponse, CommandResponse, EmptyError, InvalidResponse, MessageResponse, UsersResponse};
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
}