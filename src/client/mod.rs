use futures_util::{FutureExt, StreamExt};
mod builder;
use crate::ws_settings::WebSocketSettings;
use crate::{common, eprint_rn, print_rn};
use builder::*;
use futures_util::select;
use futures_util::stream::{SplitSink, SplitStream};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::broadcast::Receiver;
use tokio::sync::{broadcast, mpsc};
use tokio_tungstenite::tungstenite::http::Uri;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{WebSocketStream, connect_async, tungstenite};

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
        mut internal_message_rx: mpsc::Receiver<Message>,
        mut user_message_rx: Receiver<Message>,
    ) {        
        loop {
            select! {
                x1 = internal_message_rx.recv().fuse() => {
                    match x1 {
                        Some(msg) => {
                            common::async_send(&mut websocket_tx, msg).await;
                        },
                        None => {}
                    }
                },
                x2 = user_message_rx.recv().fuse() => {
                     match x2{
                        Ok(msg) => {
                            common::async_send(&mut websocket_tx, msg).await;
                        },
                        Err(err) => {
                            eprint_rn!("CLIENT :: {}", err.to_string());
                        }
                    }
                },
            }
        }
    }

    async fn spawn_client_read_handler(
        mut websocket_rx: SplitStream<WebSocketStream<impl AsyncRead + AsyncWrite + Unpin>>,
        message_tx: mpsc::Sender<Message>,
        address: Uri,
        auto_pong: bool,
        log_incoming_messages: bool,
    ) {        
        loop {
            let msg = match websocket_rx.next().await {
                Some(op) => match op {
                    Ok(msg) => msg,
                    Err(tungstenite::Error::ConnectionClosed) => {
                        eprint_rn!("CLIENT :: {} | connection closed:", address);
                        break;
                    }
                    Err(err) => {
                        eprint_rn!("CLIENT :: {} | {}", address, err.to_string());
                        break;
                    }
                },
                None => {
                    continue;
                }
            };

            if log_incoming_messages {
                print_rn!("INC [{}] << {}", address, msg)
            }

            if auto_pong && msg.is_ping() {
                message_tx.send(common::pong()).await.expect("Failed to send");
            }
        }
    }

    async fn spawn_client_connection(
        usr_msg_rx: Receiver<Message>,
        address: Uri,
        on_connect_message: Option<Message>,
        auto_pong: bool,
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
        
        let (internal_msg_tx, internal_msg_rx) = mpsc::channel::<Message>(4);

        let read_handle = tokio::spawn(async move {
            Self::spawn_client_read_handler(
                ws_rx,
                internal_msg_tx,
                address,
                auto_pong,
                log_incoming_messages,
            )
            .await;
        });

        let write_handle = tokio::spawn(async move {
            Self::spawn_client_write_handler(ws_tx, internal_msg_rx, usr_msg_rx).await;
        });

        let _ = tokio::join!(read_handle, write_handle);
    }

    pub async fn start(self) {
        let (usr_msg_tx, _) = broadcast::channel::<Message>(4);

        common::spawn_keystroke_listener(usr_msg_tx.clone(), self.settings.keybindings);

        print_rn!("CLIENT :: Connecting to: {}", self.address.to_string());

        Self::spawn_client_connection(
            usr_msg_tx.subscribe(),
            self.address,
            self.settings.on_connect_message,
            self.settings.auto_pong,
            self.settings.log_incoming_messages,
        )
        .await;
    }
}
