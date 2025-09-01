use crate::keybindings::Keybindings;
use crate::ws_settings::{WebSocketSettings, WebSocketSettingsError};
use termion::event::Key;
use tokio_tungstenite::tungstenite::Message;

pub struct WebSocketSettingsBuilder {
    parts: Result<WebSocketSettingsParts, WebSocketSettingsError>,
}

pub struct WebSocketSettingsParts {
    pub(crate) keybindings: Option<Keybindings>,
    pub(crate) on_connect_message: Option<Message>,
    pub(crate) log_incoming_messages: bool,
}

impl Default for WebSocketSettingsParts {
    fn default() -> Self {
        WebSocketSettingsParts {
            keybindings: None,
            on_connect_message: None,
            log_incoming_messages: false,
        }
    }
}

impl WebSocketSettingsBuilder {
    pub(crate) fn new() -> WebSocketSettingsBuilder {
        WebSocketSettingsBuilder {
            parts: Ok(WebSocketSettingsParts::default()),
        }
    }

    fn map<F>(self, func: F) -> Self
    where
        F: FnOnce(WebSocketSettingsParts) -> Result<WebSocketSettingsParts, WebSocketSettingsError>,
    {
        WebSocketSettingsBuilder {
            parts: self.parts.and_then(func),
        }
    }

    pub fn add_keybinding(self, key: Key, message: fn() -> Message) -> Self {
        self.map(move |mut parts| {
            parts
                .keybindings
                .get_or_insert_with(Keybindings::new)
                .add(key, message);
            Ok(parts)
        })
    }

    pub fn on_connect_message(self, message: Message) -> Self {
        self.map(move |mut parts| {
            parts.on_connect_message = Some(message);
            Ok(parts)
        })
    }
    
    pub fn log_incoming_messages(self) -> Self {
        self.map(move |mut parts| {
            parts.log_incoming_messages = true;
            Ok(parts)
        })
    }

    pub fn build(self) -> Result<WebSocketSettings, WebSocketSettingsError> {
        WebSocketSettings::from_parts(self.parts?).map_err(Into::into)
    }
}
