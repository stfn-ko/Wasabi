mod builder;
use crate::keybindings::Keybindings;
use crate::{common, eprint_rn, print_rn};
use builder::*;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite;
use tokio_tungstenite::tungstenite::http::Uri;
use tokio_tungstenite::tungstenite::{Message, connect};

#[derive(Debug)]
pub enum ClientError {
    BuilderError(String),
}

pub struct Client {
    address: Uri,
    keybindings: Arc<Keybindings>,
    on_connect_message: Option<Message>,
    ping_auto_response: bool,
    log_incoming_messages: bool,
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
            on_connect_message: src.on_connect_message,
            ping_auto_response: src.auto_pong,
            log_incoming_messages: src.log_incoming_messages,
        })
    }

    fn spawn_client_connection(
        stroke_receiver: broadcast::Receiver<Key>,
        address: Uri,
        keybindings: Arc<Keybindings>,
        on_connect_message: Option<Message>,
        auto_pong: bool,
        log_incoming_messages: bool,
    ) {
        let (mut connection, _) =
            connect(&address).expect(&format!("Failed to connect to {}", address));

        if let Some(msg) = on_connect_message {
            common::send(&mut connection, msg)
        }

        loop {
            let msg = match connection.read() {
                Ok(msg) => msg,
                Err(tungstenite::Error::ConnectionClosed) => {
                    eprint_rn!("CLIENT :: connection closed");
                    break;
                }
                Err(err) => {
                    eprint_rn!("CLIENT :: {}", err.to_string());
                    break;
                }
            };

            if log_incoming_messages {
                print_rn!("INC << {msg}");
            }

            if auto_pong && msg.is_ping() {
                let resp = common::pong();
                common::send(&mut connection, resp.clone());
                print_rn!("OUT >> {}", resp);
            }
        }
    }

    pub fn start(self) {
        let (sender, _) = broadcast::channel::<Key>(4);

        common::spawn_keystroke_listener(sender.clone());

        print_rn!("CLIENT :: Connecting to: {}", self.address.to_string());

        Self::spawn_client_connection(
            sender.subscribe(),
            self.address,
            self.keybindings.clone(),
            self.on_connect_message.clone(),
            self.ping_auto_response.clone(),
            self.log_incoming_messages.clone(),
        );
    }
}
