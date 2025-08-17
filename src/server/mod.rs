mod builder;
use crate::common_messages::pong;
use builder::*;
use std::net::TcpListener;
use std::net::{SocketAddr, TcpStream};
use std::sync::Arc;
use std::thread::spawn;
use tungstenite::{WebSocket, accept};

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
    echo_incoming_messages_to_console: bool,
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
            echo_incoming_messages_to_console: src.echo_incoming_messages_to_console,
        })
    }

    fn send(ws: &mut WebSocket<TcpStream>, msg: Message) {
        match ws.send(msg.clone()) {
            Ok(_) => println!("OUTCOMING :: {}", msg),
            Err(e) => println!("Error sending message: {:?}", e),
        }
    }

    pub fn start(self) {
        let server = TcpListener::bind(&self.address)
            .expect(&format!("Failed to bind server to {}", &self.address));

        for stream in server.incoming() {
            let keybindings = Arc::clone(&self.keybindings);
            let on_connect_message = self.on_connect_message.clone();

            spawn(move || {
                let mut websocket =
                    accept(stream.unwrap()).expect("Error during websocket connection");

                if let Some(msg) = on_connect_message {
                    Self::send(&mut websocket, msg)
                }

                loop {
                    let msg = match websocket.read() {
                        Ok(msg) => msg,
                        Err(tungstenite::Error::ConnectionClosed) => {
                            eprintln!("SERVER :: connection closed");
                            break;
                        }
                        Err(err) => {
                            eprintln!("SERVER :: {}", err.to_string());
                            break;
                        }
                    };

                    if self.echo_incoming_messages_to_console {
                        println!("INCOMING :: {}", msg)
                    }

                    if msg.is_ping() {
                        Self::send(&mut websocket, pong())
                    }
                }
            });
        }
    }
}
