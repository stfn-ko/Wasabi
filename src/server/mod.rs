mod builder;
use crate::{common, eprint_rn, print_rn};
use builder::*;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{FutureExt, SinkExt, StreamExt, select};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync;
use tokio::sync::broadcast::Receiver;
use tokio::sync::{broadcast, mpsc};
use tokio_tungstenite::WebSocketStream;
use tungstenite::Error as tError;
use tungstenite::Message;

#[derive(Debug)]
pub enum Error {
    BuilderError,
    ServerError,
}

pub struct Server {
    address: SocketAddr,

    #[allow(dead_code)]
    keybindings: Arc<Keybindings>,

    on_connect_message: Option<Message>,

    // flags
    auto_pong: bool,
    log_incoming_messages: bool,
}

impl Server {
    pub fn builder() -> ServerBuilder {
        ServerBuilder::new()
    }

    fn from_parts(src: ServerParts) -> Result<Server, Error> {
        if src.address.is_none() {
            return Err(Error::BuilderError);
        }

        Ok(Server {
            address: src.address.unwrap(),
            keybindings: Arc::from(src.keybindings),
            on_connect_message: src.on_connect_message,
            auto_pong: src.auto_pong,
            log_incoming_messages: src.log_incoming_messages,
        })
    }

    async fn spawn_server_read_handler(
        mut websocket_rx: SplitStream<WebSocketStream<tokio::net::TcpStream>>,
        message_tx: mpsc::Sender<Message>,
        auto_pong: bool,
        log_incoming_messages: bool,
    ) {
        loop {
            let msg = match websocket_rx.next().await {
                Some(op) => match op {
                    Ok(msg) => msg,
                    Err(tError::ConnectionClosed) => {
                        eprint_rn!("SERVER :: connection closed");
                        break;
                    }
                    Err(err) => {
                        eprint_rn!("SERVER :: {}", err.to_string());
                        break;
                    }
                },
                None => {
                    continue;
                }
            };

            if log_incoming_messages {
                print_rn!("INCOMING :: {}", msg)
            }

            if auto_pong && msg.is_ping(){
                message_tx.send(msg).await.expect("Failed to send");
            }
        }
    }

    async fn spawn_server_write_handler(
        mut websocket_tx: SplitSink<WebSocketStream<tokio::net::TcpStream>, Message>,
        mut message_rx: mpsc::Receiver<Message>,
        mut keystroke_rx: Receiver<Key>,
        keybindings: Arc<Keybindings>,
    ) {
        loop {
            select! {
                x1 = keystroke_rx.recv().fuse() => {
                     match x1{
                        Ok(key) => {
                            print_rn!("SERVER :: KEYSTROKE :: {:?}", key);

                            if let Some(kb) = keybindings.at(key) {
                                let msg = kb();
                                websocket_tx
                                    .send(msg.clone())
                                    .await
                                    .expect("Failed to send");
                                print_rn!("OUTCOMING :: {}", msg);
                            }
                        },
                        Err(err) => {
                            eprint_rn!("SERVER :: {}", err.to_string());
                        }
                    }
                },
                x2 = message_rx.recv().fuse() => {
                    match x2 {
                        Some(msg) => {
                            websocket_tx.send(msg.clone()).await.expect("Failed to send");
                            print_rn!("OUTCOMING :: {}", msg);
                        },
                        None => {}
                    }
                },
            }
        }
    }

    pub async fn start(self) {
        let (keystroke_tx, _) = broadcast::channel::<Key>(4);

        common::spawn_keystroke_listener(keystroke_tx.clone());

        eprint_rn!("SERVER :: Starting at: {}", self.address.to_string());

        let try_socket = TcpListener::bind(&self.address).await;
        let server = try_socket.expect("Failed to bind ");

        while let Ok((stream, _)) = server.accept().await {
            let addr = stream
                .peer_addr()
                .expect("Connected streams should have a peer address");

            let ws_stream = tokio_tungstenite::accept_async(stream)
                .await
                .expect("Error during the websocket handshake occurred");

            print_rn!("SERVER :: New connection: {addr}");

            let (mut ws_tx, ws_rx) = ws_stream.split();

            let (msg_tx, msg_rx) = mpsc::channel::<Message>(4);

            let keybindings = self.keybindings.clone();

            let keystroke_rx = keystroke_tx.subscribe();

            if let Some(msg) = self.on_connect_message.clone() {
                ws_tx.send(msg).await.expect("Failed to send");
            }

            tokio::spawn(async move {
                Self::spawn_server_read_handler(
                    ws_rx,
                    msg_tx,
                    self.auto_pong,
                    self.log_incoming_messages,
                )
                .await;
            });

            tokio::spawn(async move {
                Self::spawn_server_write_handler(ws_tx, msg_rx, keystroke_rx, keybindings).await;
            });
        }
    }
}
