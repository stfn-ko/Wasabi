use crate::client::{Client, ClientError};
use crate::ws_settings::WebSocketSettings;
pub(crate) use tokio_tungstenite::tungstenite::http::Uri;

pub struct ClientBuilder {
    parts: Result<ClientParts, ClientError>,
}

pub struct ClientParts {
    pub(crate) address: Option<Uri>,
    pub(crate) settings: WebSocketSettings,
}

impl Default for ClientParts {
    fn default() -> Self {
        ClientParts {
            address: None,
            settings: WebSocketSettings::default(),
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

    pub fn settings(self, settings: WebSocketSettings) -> Self {
        self.map(move |mut parts| {
            parts.settings = settings;
            Ok(parts)
        })
    }

    pub fn build(self) -> Result<Client, ClientError> {
        Client::from_parts(self.parts?).map_err(Into::into)
    }
}
