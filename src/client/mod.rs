use futures_util::StreamExt;
mod builder;
use crate::{common, eprint_rn, print_rn, WebSocketSettings};
use builder::*;
use futures_util::stream::{SplitSink, SplitStream};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::broadcast;
use tokio::sync::broadcast::Receiver;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::http::Uri;
use tokio_tungstenite::{WebSocketStream, connect_async};

#[derive(Debug)]
pub enum ClientError {
    BuilderError(String),
}

pub struct Client {
    address: Uri,
    settings: WebSocketSettings,
}

impl Client {
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    fn from_parts(src: ClientParts) -> Result<Client, ClientError> {
        if src.address.is_none() {
            return Err(ClientError::BuilderError(
                "address not specified".to_string(),
            ));
        }

        Ok(Client {
            address: src.address.unwrap(),
            settings: src.settings,
        })
    }

    async fn spawn_client_write_handler(
        mut websocket_tx: SplitSink<WebSocketStream<impl AsyncRead + AsyncWrite + Unpin>, Message>,
        mut user_message_rx: Receiver<Message>,
    ) {
        loop {
            match user_message_rx.recv().await {
                Ok(msg) => {
                    common::async_send(&mut websocket_tx, msg).await;
                }
                Err(err) => {
                    eprint_rn!("CLIENT :: {}", err.to_string());
                }
            }
        }
    }

    async fn spawn_client_read_handler(
        mut websocket_rx: SplitStream<WebSocketStream<impl AsyncRead + AsyncWrite + Unpin>>,
        address: Uri,
        log_incoming_messages: bool,
    ) {
        while let Some(msg) = websocket_rx.next().await {
            let msg = match msg {
                Ok(msg) => msg,
                Err(err) => {
                    eprint_rn!("CLIENT ERROR :: {} | {}", address, err);
                    break;
                }
            };

            if log_incoming_messages {
                match msg {
                    Message::Pong(_) => {
                        print_rn!("[{}] INC << pong", address)
                    }
                    Message::Text(t) => {
                        print_rn!("[{}] INC << {}", address, t)
                    }
                    _ => {}
                }
            }
        }
    }

    async fn spawn_client_connection(
        usr_msg_rx: Receiver<Message>,
        address: Uri,
        on_connect_message: Option<Message>,
        log_incoming_messages: bool,
    ) {
        let (connection, _) = match connect_async(&address).await {
            Ok(connection) => connection,
            Err(e) => {
                eprint_rn!("CLIENT :: Failed to connect: {e}");
                return;
            }
        };

        let (mut ws_tx, ws_rx) = connection.split();

        if let Some(msg) = on_connect_message {
            common::async_send(&mut ws_tx, msg).await;
        }

        let read_handle = tokio::spawn(async move {
            Self::spawn_client_read_handler(ws_rx, address, log_incoming_messages).await;
        });

        let write_handle = tokio::spawn(async move {
            Self::spawn_client_write_handler(ws_tx, usr_msg_rx).await;
        });

        let _ = tokio::try_join!(read_handle, write_handle);
    }

    pub async fn start(self) {
        let (usr_msg_tx, _) = broadcast::channel::<Message>(4);

        common::spawn_keystroke_listener(usr_msg_tx.clone(), self.settings.keybindings);

        print_rn!("CLIENT :: Connecting to: {}", self.address.to_string());

        Self::spawn_client_connection(
            usr_msg_tx.subscribe(),
            self.address,
            self.settings.on_connect_message,
            self.settings.log_incoming_messages,
        )
        .await;
    }
}
