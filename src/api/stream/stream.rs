use std::collections::HashMap;
use std::time::Duration;

use async_tungstenite::{
    tokio::connect_async,
    tungstenite::{self, Message},
};
use chrono::Local;
use futures::prelude::*;
use tokio::{
    select,
    sync::{mpsc, oneshot},
    time::sleep,
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, trace, warn};

use crate::api::stream::errors::ChatConnectError;
use crate::api::stream::errors::ChatMessageStreamError;
use crate::api::stream::structs::{ChatMessage, ChatSocketMessage};
use crate::utils::utils::random_string;

const CHAT_MESSAGES_BUFFER: usize = 32;
const DEFAULT_PING_INTERVAL: Duration = Duration::from_secs(30);

// A stream of chat messages
#[derive(Debug)]
pub struct ChatMessageStream {
    cancellation_token: CancellationToken,
    messages: mpsc::Receiver<Result<ChatMessage, ChatMessageStreamError>>,
}

impl ChatMessageStream {
    // Connect to trovo chat using the given chat token.
    // FIXME: Sometimes connecting takes too much time and then crashes WebSocket(Protocol(HandshakeIncomplete))
    pub async fn connect(chat_token: String) -> Result<ChatMessageStream, ChatConnectError> {
        let cancellation_token = CancellationToken::new();

        let (ws_stream, _) = connect_async("wss://open-chat.trovo.live/chat").await?;
        let (mut writer, reader) = ws_stream.split();
        let (
            socket_messages_sender,
            socket_messages_receiver
        ) = mpsc::channel(1);
        let (
            chat_messages_sender,
            chat_messages_receiver
        ) = mpsc::channel(CHAT_MESSAGES_BUFFER);
        let (
            auth_response_sender,
            auth_response_receiver
        ) = oneshot::channel();

        let auth_nonce = random_string(32).await;

        let reader = SocketMessagesReader {
            reader,
            cancellation_token: cancellation_token.clone(),
            auth: (auth_nonce.clone(), Some(auth_response_sender)),
            chat_messages_sender: chat_messages_sender.clone(),
            ping: Default::default(),
        };
        reader.spawn();

        let msg = serde_json::to_string(&ChatSocketMessage::Auth {
            nonce: auth_nonce,
            data: HashMap::from_iter([("token".to_string(), chat_token)]),
        })?;
        writer.send(msg.into()).await?;

        auth_response_receiver
            .await
            .map_err(|_| ChatConnectError::SocketClosed)??;

        let writer = SocketMessagesWriter {
            writer,
            cancellation_token: cancellation_token.clone(),
            socket_messages_receiver,
            chat_messages_sender,
        };
        writer.spawn();

        let pinger = Pinger {
            ping: Default::default(),
            socket_messages_sender,
        };
        pinger.spawn();

        Ok(ChatMessageStream {
            cancellation_token,
            messages: chat_messages_receiver,
        })
    }

    // Close the chat socket, causing any further calls to `next()` to return `None`.
    //
    // Automatically called on drop. Calling multiple times has no effect.
    pub fn close(&self) {
        self.cancellation_token.cancel()
    }
}

impl Stream for ChatMessageStream {
    type Item = Result<ChatMessage, ChatMessageStreamError>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.messages.poll_recv(cx)
    }
}

impl Drop for ChatMessageStream {
    fn drop(&mut self) {
        self.close()
    }
}

#[derive(Debug, PartialEq)]
enum Continuation {
    Continue,
    Stop,
}

#[derive(Debug)]
struct Ping {
    interval: Duration,
    iteration: u64,

    // The last iteration that we got a Pong response to
    acknowledged: u64,
}

impl Default for Ping {
    fn default() -> Self {
        Self {
            interval: DEFAULT_PING_INTERVAL,
            iteration: 0,
            acknowledged: 0,
        }
    }
}

// TODO: Dynamic state provided by 'SocketMessagesReader::handle_socket_message'
#[derive(Debug)]
struct Pinger {
    ping: Ping,
    socket_messages_sender: mpsc::Sender<ChatSocketMessage>,
}

impl Pinger {
    fn spawn(mut self) {
        tokio::spawn(async move {
            loop {
                sleep(self.ping.interval).await;
                println!("-------------Ping sent at {}-------------", Local::now());
                self.ping.iteration += 1;

                let msg = ChatSocketMessage::Ping { nonce: self.ping.iteration.to_string() };
                trace!(?msg, "sending ping");
                match self.socket_messages_sender.send(msg).await {
                    Err(_) => panic!("Service unavailable: cannot send ping"),
                    _ => {}
                };
            };
        });
    }
}


struct SocketMessagesReader<R> {
    cancellation_token: CancellationToken,
    reader: R,
    chat_messages_sender: mpsc::Sender<Result<ChatMessage, ChatMessageStreamError>>,
    auth: (
        String,
        Option<oneshot::Sender<Result<(), ChatConnectError>>>,
    ),
    ping: Ping,
}

impl<R> SocketMessagesReader<R>
    where
        R: 'static + Stream<Item=Result<Message, tungstenite::Error>> + Send + Unpin,
{
    fn spawn(mut self) {
        tokio::spawn(async move {
            loop {
                match self.next().await {
                    Ok(Continuation::Stop) => {
                        trace!("Socket reader exited gracefully");
                        break;
                    }
                    Err(err) => {
                        error!(?err, "Socket reader error");
                        self.chat_messages_sender.send(Err(err)).await.ok();
                        break;
                    }
                    _ => {}
                }
            }
        });
    }

    async fn next(&mut self) -> Result<Continuation, ChatMessageStreamError> {
        select! {
            _ = self.cancellation_token.cancelled() => {
                Ok(Continuation::Stop)
            }
            Some(msg) = self.reader.next() => {
                self.handle_message(msg?).await
            }
            else => {
                Ok(Continuation::Stop)
            }
        }
    }

    async fn handle_message(
        &mut self,
        msg: Message,
    ) -> Result<Continuation, ChatMessageStreamError> {
        trace!(?msg, "Incoming websocket message");

        match msg {
            Message::Text(text) => {
                let msg: ChatSocketMessage = serde_json::from_str(&text)?;
                Ok(self.handle_socket_message(msg).await)
            }
            Message::Binary(bytes) => {
                let msg = serde_json::from_slice(bytes.as_slice())?;
                Ok(self.handle_socket_message(msg).await)
            }
            Message::Ping(_) => todo!(),
            Message::Pong(_) => todo!(),
            Message::Close(reason) => Err(
                ChatMessageStreamError::SocketClosed(reason)
            ),
            Message::Frame(_) => { Ok(Continuation::Continue) }
        }
    }


    async fn handle_socket_message(&mut self, msg: ChatSocketMessage) -> Continuation {
        debug!( ? msg, "Incoming chat socket message");
        match msg {
            ChatSocketMessage::Response { nonce } => {
                if self.auth.0 == nonce {
                    if let Some(auth) = self.auth.1.take() {
                        auth.send(Ok(())).ok();
                    }
                }
                Continuation::Continue
            }
            ChatSocketMessage::Pong { nonce, data } => {
                let iteration: u64 = match nonce.parse() {
                    Ok(v) => v,
                    Err(err) => {
                        warn!( ? err, "Failed to parse pong nonce as u64, ignoring...");
                        return Continuation::Continue;
                    }
                };
                debug!( ?iteration, "Received pong");
                // Ignore potentially delayed responses from any old pings
                if iteration > self.ping.acknowledged {
                    self.ping.acknowledged = iteration;
                    self.ping.interval = Duration::from_secs(data.gap);
                }
                Continuation::Continue
            }
            ChatSocketMessage::Chat {
                channel_info: _,
                data,
            } => {
                for chat in data.chats {
                    if self.chat_messages_sender.send(Ok(chat)).await.is_err() {
                        // Messages receiver must have been dropped and so we just need to cleanup
                        return Continuation::Stop;
                    }
                }
                Continuation::Continue
            }
            _ => unreachable!(),
        }
    }
}

impl<R> Drop for SocketMessagesReader<R> {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
    }
}

struct SocketMessagesWriter<W> {
    cancellation_token: CancellationToken,
    writer: W,
    socket_messages_receiver: mpsc::Receiver<ChatSocketMessage>,
    chat_messages_sender: mpsc::Sender<Result<ChatMessage, ChatMessageStreamError>>,
}

impl<W> SocketMessagesWriter<W>
    where
        W: 'static + Sink<Message, Error=tungstenite::Error> + Send + Unpin,
{
    fn spawn(mut self) {
        tokio::spawn(async move {
            loop {
                match self.next().await {
                    Ok(Continuation::Stop) => {
                        trace!("Socket writer exited gracefully");
                        break;
                    }
                    Err(err) => {
                        error!(?err, "Socket writer error");
                        self.chat_messages_sender.send(Err(err)).await.ok();
                        break;
                    }
                    _ => {}
                }
            }
        });
    }

    async fn next(&mut self) -> Result<Continuation, ChatMessageStreamError> {
        select! {
            _ = self.cancellation_token.cancelled() => {
                Ok(Continuation::Stop)
            }
            Some(msg) = self.socket_messages_receiver.recv() => {
                self.handle_message(msg).await?;
                Ok(Continuation::Continue)
            }
            else => {
                Ok(Continuation::Stop)
            }
        }
    }

    async fn handle_message(
        &mut self,
        msg: ChatSocketMessage,
    ) -> Result<(), ChatMessageStreamError> {
        trace!(?msg, "Outgoing websocket message");
        let msg = serde_json::to_string(&msg)?;
        self.writer.send(msg.into()).await?;
        Ok(())
    }
}

impl<W> Drop for SocketMessagesWriter<W> {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
    }
}
