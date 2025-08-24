mod builder;
use crate::ws_settings::WebSocketSettings;
use crate::{common, eprint_rn, print_rn, tungstenite};
use builder::*;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{FutureExt, SinkExt, StreamExt, select};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::Receiver;
use tokio::sync::{broadcast, mpsc};
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug)]
pub enum Error {
    BuilderError,
    ServerError,
}

pub struct Server {
    address: SocketAddr,
    settings: WebSocketSettings,
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
            settings: src.settings,
        })
    }

    async fn spawn_server_read_handler(
        mut websocket_rx: SplitStream<WebSocketStream<TcpStream>>,
        message_tx: mpsc::Sender<Message>,
        address: SocketAddr,
        auto_pong: bool,
        log_incoming_messages: bool,
    ) {
        loop {
            let msg = match websocket_rx.next().await {
                Some(op) => match op {
                    Ok(msg) => msg,
                    Err(tungstenite::Error::ConnectionClosed) => {
                        eprint_rn!("SERVER :: {} | connection closed:", address);
                        break;
                    }
                    Err(err) => {
                        eprint_rn!("SERVER :: {} | {}", address, err.to_string());
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
                message_tx
                    .send(common::pong())
                    .await
                    .expect("Failed to send");
            }
        }
    }

    async fn spawn_server_write_handler(
        mut websocket_tx: SplitSink<WebSocketStream<TcpStream>, Message>,
        mut internal_message_rx: mpsc::Receiver<Message>,
        mut user_message_rx: Receiver<Message>,
    ) {
        loop {
            select! {
                x1 = user_message_rx.recv().fuse() => {
                     match x1{
                        Ok(msg) => {
                            common::async_send(&mut websocket_tx, msg).await;
                        },
                        Err(err) => {
                            eprint_rn!("SERVER :: {}", err.to_string());
                        }
                    }
                },
                x2 = internal_message_rx.recv().fuse() => {
                    match x2 {
                        Some(msg) => {
                            common::async_send(&mut websocket_tx, msg).await;
                        },
                        None => {}
                    }
                },
            }
        }
    }

    async fn start_new_connection(
        stream: TcpStream,
        usr_msg_rx: Receiver<Message>,
        on_connect_message: Option<&Message>,
        auto_pong: bool,
        log_incoming_messages: bool,
    ) {
        let addr = stream
            .peer_addr()
            .expect("Connected streams should have a peer address");

        let ws_stream = tokio_tungstenite::accept_async(stream)
            .await
            .expect("Error during the websocket handshake occurred");

        print_rn!("SERVER :: New connection: {addr}");

        let (mut ws_tx, ws_rx) = ws_stream.split();

        let (internal_msg_tx, internal_msg_rx) = mpsc::channel::<Message>(4);

        if let Some(msg) = on_connect_message {
            ws_tx.send(msg.clone()).await.expect("Failed to send");
            print_rn!("OUT >> {}", msg);
        }

        tokio::spawn(async move {
            Self::spawn_server_read_handler(
                ws_rx,
                internal_msg_tx,
                addr,
                auto_pong,
                log_incoming_messages,
            )
            .await;
        });

        tokio::spawn(async move {
            Self::spawn_server_write_handler(ws_tx, internal_msg_rx, usr_msg_rx).await;
        });
    }

    pub async fn start(self) {
        let (usr_msg_tx, _) = broadcast::channel::<Message>(4);

        common::spawn_keystroke_listener(usr_msg_tx.clone(), self.settings.keybindings);

        print_rn!("SERVER :: Starting at: {}", self.address.to_string());

        let try_socket = TcpListener::bind(&self.address).await;
        let server = try_socket.expect("Failed to bind ");

        while let Ok((stream, _)) = server.accept().await {
            Self::start_new_connection(
                stream,
                usr_msg_tx.subscribe(),
                self.settings.on_connect_message.as_ref(),
                self.settings.auto_pong,
                self.settings.log_incoming_messages,
            )
            .await;
        }
    }
}
