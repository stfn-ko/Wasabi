mod client;
mod common;
mod keybindings;
mod server;
mod ws_settings;

pub use crate::client::*;
pub use crate::server::*;
pub use crate::ws_settings::*;
pub use std::net::SocketAddr;
pub use termion::event::Key;
pub use tokio;
pub use tokio_tungstenite::tungstenite;
pub use tokio_tungstenite::tungstenite::{Bytes, Message, Utf8Bytes};

/*
 TODO: 
 Fix this on both client and server receiving side:
    [2025-08-24T15:22:52+03:00] KEY :: Char('p')
    [2025-08-24T15:22:52+03:00] OUT >> ping
    [2025-08-24T15:22:52+03:00] INC [127.0.0.1:62644] << ping
    [2025-08-24T15:22:52+03:00] INC [127.0.0.1:62644] << pong 
*/