pub(crate) use crate::server::{Error, Server};
use crate::ws_settings::WebSocketSettings;
pub(crate) use std::net::SocketAddr;

pub struct ServerBuilder {
    parts: Result<ServerParts, Error>,
}

pub struct ServerParts {
    pub(crate) address: Option<SocketAddr>,
    pub(crate) settings: WebSocketSettings,
}

impl Default for ServerParts {
    fn default() -> Self {
        ServerParts {
            address: None,
            settings: WebSocketSettings::default(),
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

    pub fn address(self, address: &str) -> Self {
        let socket_addr = address
            .parse::<SocketAddr>()
            .expect("Failed to set address");

        self.map(move |mut parts| {
            parts.address = Some(socket_addr);
            Ok(parts)
        })
    }

    pub fn settings(self, settings: WebSocketSettings) -> Self {
        self.map(move |mut parts| {
            parts.settings = settings;
            Ok(parts)
        })
    }

    pub fn build(self) -> Result<Server, Error> {
        Server::from_parts(self.parts?).map_err(Into::into)
    }
}
