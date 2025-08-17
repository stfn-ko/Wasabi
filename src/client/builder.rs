pub(crate) use crate::keybindings::*;
pub(crate) use tungstenite::http::Uri;
use crate::client::{Client, ClientError};

pub struct ClientBuilder {
    parts: Result<ClientParts, ClientError>,
}

pub struct ClientParts {
    pub(crate) address: Option<Uri>,

    pub(crate) keybindings: Keybindings,

    pub(crate) echo_incoming_messages_to_console: bool,
}

impl Default for ClientParts {
    fn default() -> Self {
        ClientParts {
            address: None,
            keybindings: Keybindings::new(),
            echo_incoming_messages_to_console: false,
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

    pub fn address(self, address: Uri) -> Self {
        assert_eq!(address.scheme_str(), Some("ws"));
        
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

    pub fn echo_messages_to_console(self) -> Self {
        self.map(move |mut parts| {
            parts.echo_incoming_messages_to_console = true;
            Ok(parts)
        })
    }

    pub fn build(self) -> Result<Client, ClientError> {
        Client::from_parts(self.parts?).map_err(Into::into)
    }
}
