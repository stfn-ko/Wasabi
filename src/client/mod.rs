mod builder;
use crate::keybindings::Keybindings;
use builder::*;
use std::sync::Arc;
use tungstenite::connect;

#[derive(Debug)]
pub enum ClientError {
    BuilderError(String),
}

pub struct Client {
    address: Uri,

    #[allow(dead_code)]
    keybindings: Arc<Keybindings>,

    // flags
    echo_incoming_messages_to_console: bool,
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
            keybindings: Arc::from(src.keybindings),
            echo_incoming_messages_to_console: src.echo_incoming_messages_to_console,
        })
    }

    pub fn start(self) {
        let (mut websocket, _) = connect(&self.address)
            .expect(&format!("Failed to connect client to {}", &self.address));

        loop {
            match websocket.read() {
                Ok(msg) => {
                    if self.echo_incoming_messages_to_console {
                        println!("INCOMING :: {msg}");
                    }
                }
                Err(tungstenite::Error::ConnectionClosed) => {
                    eprintln!("CLIENT :: connection closed");
                    break;
                }
                Err(err) => {
                    eprintln!("CLIENT :: {}", err.to_string());
                    break;
                }
            };
        }
    }
}
