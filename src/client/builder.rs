pub(crate) use crate::keybindings::*;
pub(crate) use tungstenite::http::Uri;
use crate::client::{Client, ClientError};

pub struct ClientBuilder {
    parts: Result<ClientParts, ClientError>,
}

pub struct ClientParts {
    pub(crate) address: Option<Uri>,
    pub(crate) keybindings: Keybindings,
    pub(crate) on_connect_message: Option<Message>,
    pub(crate) auto_pong: bool,
    pub(crate) log_incoming_messages: bool,
}

impl Default for ClientParts {
    fn default() -> Self {
        ClientParts {
            address: None,
            keybindings: Keybindings::new(),
            on_connect_message: None,
            auto_pong: false,
            log_incoming_messages: false,
        }
    }
}

impl ClientBuilder {
    pub(crate) fn new() -> ClientBuilder {
        ClientBuilder {
            parts: Ok(ClientParts::default()),
        }
    }

    fn map<F>(self, func: F) -> Self
    where
        F: FnOnce(ClientParts) -> Result<ClientParts, ClientError>,
    {
        ClientBuilder {
            parts: self.parts.and_then(func),
        }
    }

    pub fn address(self, address: &str) -> Self {
        let uri = address.parse::<Uri>().expect("Failed to set address");
        assert_eq!(uri.scheme_str(), Some("ws"));
        
        self.map(move |mut parts| {
            parts.address = Some(uri);
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

    pub fn auto_pong(self) -> Self {
        self.map(move |mut parts| {
            parts.auto_pong = true;
            Ok(parts)
        })
    }

    pub fn log_incoming_messages(self) -> Self {
        self.map(move |mut parts| {
            parts.log_incoming_messages = true;
            Ok(parts)
        })
    }

    pub fn build(self) -> Result<Client, ClientError> {
        Client::from_parts(self.parts?).map_err(Into::into)
    }
}
