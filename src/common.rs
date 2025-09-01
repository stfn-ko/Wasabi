use crate::keybindings::Keybindings;
use futures_util::SinkExt;
use futures_util::stream::SplitSink;
use std::thread;
use termion::event::Key;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::broadcast;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::{Bytes, Message, Utf8Bytes};

#[macro_export]
macro_rules! print_rn {
    ($($arg:tt)*) => ({
        print!("\r\n[{}] {}\r\n", chrono::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, false) , format!($($arg)*));
    })
}

#[macro_export]
macro_rules! eprint_rn {
    ($($arg:tt)*) => ({
        eprint!("\r\n[{}] {}\r\n", chrono::Local::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, false) , format!($($arg)*));
    })
}

#[inline]
pub fn ping() -> Message {
    Message::Ping(Bytes::new())
}

#[inline]
pub fn close() -> Message {
    Message::Close(Some(CloseFrame {
        code: CloseCode::Normal,
        reason: Utf8Bytes::from("connection close"),
    }))
}

pub(crate) async fn async_send(
    ws: &mut SplitSink<WebSocketStream<impl AsyncRead + AsyncWrite + Unpin>, Message>,
    msg: Message,
) {
    if let Err(e) = ws.send(msg.clone()).await {
        eprint_rn!("Error sending message: {:?}", e);
        return;
    }

    match msg {
        Message::Text(text) => print_rn!("OUT >> {text}"),
        Message::Ping(_) => print_rn!("OUT >> ping"),
        Message::Pong(_) => print_rn!("OUT >> pong"),
        _ => {}
    }
}

pub(crate) fn spawn_keystroke_listener(
    sender: broadcast::Sender<Message>,
    keybindings: Option<Keybindings>,
) {
    use std::io::{Write, stdin, stdout};
    use termion::clear;
    use termion::cursor;
    use termion::input::TermRead;
    use termion::raw::IntoRawMode;

    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    write!(
        stdout,
        "{}{}{}",
        clear::BeforeCursor,
        cursor::Hide,
        cursor::Goto(1, 1)
    )
    .unwrap();
    stdout.flush().unwrap();

    thread::spawn(move || {
        for k in stdin.keys() {
            match k {
                Ok(Key::Ctrl('c')) => return, // todo: sigkill application
                Ok(k) => {
                    if sender.receiver_count() > 0 && keybindings.is_some() {
                        if let Some(msg_cb) = keybindings.as_ref().unwrap().at(k) {
                            sender.send(msg_cb()).expect("Error sending message");
                        }
                    }

                    print_rn!("KEY :: {:?}\r", k);
                }
                Err(e) => eprint_rn!("Error: {:?}", e),
            }

            stdout.flush().unwrap();
        }
    });
}
