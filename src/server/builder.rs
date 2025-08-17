pub(crate) use crate::keybindings::*;
pub(crate) use crate::server::{Error, Server};
pub(crate) use std::net::SocketAddr;
pub(crate) use tungstenite::Message;

pub struct ServerBuilder {
    parts: Result<ServerParts, Error>,
}

pub struct ServerParts {
    pub(crate) address: Option<SocketAddr>,

    pub(crate) keybindings: Keybindings,

    pub(crate) on_connect_message: Option<Message>,

    pub(crate) echo_incoming_messages_to_console: bool,
}

impl Default for ServerParts {
    fn default() -> Self {
        ServerParts {
            address: None,
            keybindings: Keybindings::new(),
            on_connect_message: None,
            echo_incoming_messages_to_console: false,
        }
    }
}

impl ServerBuilder {
    pub(crate) fn new() -> ServerBuilder {
        ServerBuilder {
            parts: Ok(ServerParts::default()),
        }
    }

    fn map<F>(self, func: F) -> Self
    where
        F: FnOnce(ServerParts) -> Result<ServerParts, Error>,
    {
        ServerBuilder {
            parts: self.parts.and_then(func),
        }
    }

    pub fn address(self, address: SocketAddr) -> Self {
        self.map(move |mut parts| {
            parts.address = Some(address);
            Ok(parts)
        })
    }

    pub fn add_keybinding(self, key: Key, message: fn() -> Message) -> Self {
        self.map(move |mut parts| {
            parts.keybindings.add(key, message);
            Ok(parts)
        })
    }

    pub fn on_connect_message(self, message: Message) -> Self {
        self.map(move |mut parts| {
            parts.on_connect_message = Some(message);
            Ok(parts)
        })
    }
    
    pub fn echo_messages_to_console(self) -> Self {
        self.map(move |mut parts| {
            parts.echo_incoming_messages_to_console = true;
            Ok(parts)
        })
    }

    pub fn build(self) -> Result<Server, Error> {
        Server::from_parts(self.parts?).map_err(Into::into)
    }
}
