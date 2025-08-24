mod builder;
use crate::keybindings::Keybindings;
use crate::ws_settings::builder::{WebSocketSettingsBuilder, WebSocketSettingsParts};
use tokio_tungstenite::tungstenite::Message;

#[derive(Debug)]
pub enum WebSocketSettingsError {
    BuilderError(String),
    PartsError(String),
}

pub struct WebSocketSettings {
    pub keybindings: Option<Keybindings>,
    pub on_connect_message: Option<Message>,
    pub auto_pong: bool,
    pub log_incoming_messages: bool,
}

impl WebSocketSettings {
    pub fn builder() -> WebSocketSettingsBuilder {
        WebSocketSettingsBuilder::new()
    }

    fn from_parts(
        src: WebSocketSettingsParts,
    ) -> Result<WebSocketSettings, WebSocketSettingsError> {
        Ok(WebSocketSettings {
            keybindings: src.keybindings,
            on_connect_message: src.on_connect_message,
            auto_pong: src.auto_pong,
            log_incoming_messages: src.log_incoming_messages,
        })
    }
}

impl Default for WebSocketSettings {
    fn default() -> Self {
        Self {
            keybindings: None,
            on_connect_message: None,
            auto_pong: false,
            log_incoming_messages: false,
        }
    }
}
