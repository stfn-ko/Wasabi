mod builder;
use crate::ws_settings::WebSocketSettings;
use crate::{common, eprint_rn, print_rn};
use builder::*;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio::sync::broadcast::Receiver;
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
        address: SocketAddr,
        log_incoming_messages: bool,
    ) {
        while let Some(msg) = websocket_rx.next().await {
            let msg = match msg {
                Ok(msg) => msg,
                Err(err) => {
                    eprint_rn!("SERVER ERROR :: {} | {}", address, err);
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

    async fn spawn_server_write_handler(
        mut websocket_tx: SplitSink<WebSocketStream<TcpStream>, Message>,
        mut user_message_rx: Receiver<Message>,
    ) {
        loop {
            match user_message_rx.recv().await {
                Ok(msg) => {
                    common::async_send(&mut websocket_tx, msg).await;
                }
                Err(err) => {
                    eprint_rn!("SERVER :: {}", err.to_string());
                }
            }
        }
    }

    async fn start_new_connection(
        stream: TcpStream,
        usr_msg_rx: Receiver<Message>,
        on_connect_message: Option<&Message>,
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

        if let Some(msg) = on_connect_message {
            ws_tx.send(msg.clone()).await.expect("Failed to send");
            print_rn!("OUT >> {}", msg);
        }

        let read_handle = tokio::spawn(async move {
            Self::spawn_server_read_handler(ws_rx, addr, log_incoming_messages).await;
        });

        let write_handle = tokio::spawn(async move {
            Self::spawn_server_write_handler(ws_tx, usr_msg_rx).await;
        });

        let _ = tokio::try_join!(read_handle, write_handle);
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
                self.settings.log_incoming_messages,
            )
            .await;
        }
    }
}
