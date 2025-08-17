use tungstenite::{Message, Utf8Bytes};
use tungstenite::protocol::CloseFrame;
use tungstenite::protocol::frame::coding::CloseCode;
use crate::Bytes;

#[inline]
pub fn ping() -> Message {
    Message::Ping(Bytes::from("ping"))
}

#[inline]
pub fn pong() -> Message {
    Message::Pong(Bytes::from("pong"))
}

#[inline]
pub fn close() -> Message {
    Message::Close(Some(CloseFrame {
        code: CloseCode::Normal,
        reason: Utf8Bytes::from("connection close"),
    }))
}