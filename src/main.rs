use chrono::{Local, Utc};
use futures::StreamExt;

use trovo_chatbot::api::client::API;
use trovo_chatbot::utils::config::CONFIG;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut api = API::new().await;
    let user = api.get_users(vec![CONFIG.target_channel_username.clone()]).await?;
    let target_channel_id = user.users.get(0).unwrap().channel_id;
    let bot_user = api.get_user_info().await?;  // me

    let mut messages = api.chat_messages_for_channel(target_channel_id).await?;

    let start_time = Utc::now().timestamp();
    let mut skipped_messages = 0;
    let mut already_skipped = false;

    while let Some(msg) = messages.next().await {
        let msg = msg?;
        // Trovo API returns messages which sent before program start too, ignore them
        if !already_skipped {
            if start_time > msg.send_time {
                skipped_messages += 1;
                continue;
            } else {
                already_skipped = true;
                println!("Skipped {} messages", skipped_messages);
            }
        };
        // Ignore messages sent by me
        if msg.sender_id != None {
            if msg.sender_id.unwrap() == bot_user.channel_id {
                continue;
            }
        }
        println!("[{}] {{{}}} {}", Local::now(), msg.nick_name, msg.content);
        api.send(msg.content, target_channel_id).await?;
    }
    Ok(())
}
