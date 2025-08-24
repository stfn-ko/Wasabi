use std::thread;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::{Bytes, Message, Utf8Bytes, WebSocket};
use termion::event::Key;
use tokio::sync::broadcast;

#[macro_export]
macro_rules! print_rn {
    ($($arg:tt)*) => ({
        print!("\r\n{}\r\n", format!($($arg)*));
    })
}

#[macro_export]
macro_rules! eprint_rn {
    ($($arg:tt)*) => ({
        eprint!("\r\n{}\r\n", format!($($arg)*));
    })
}

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

pub(crate) fn send<StreamType>(ws: &mut WebSocket<StreamType>, msg: Message)
where
    StreamType: std::io::Read,
    StreamType: std::io::Write,
{
    match ws.send(msg.clone()) {
        Ok(_) => print_rn!("OUTCOMING :: {}", msg),
        Err(e) => eprint_rn!("Error sending message: {:?}", e),
    }
}

pub(crate) fn spawn_keystroke_listener(sender: broadcast::Sender<Key>) {
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
                    if sender.receiver_count() > 0 {
                        let _ = sender.send(k.clone());
                    }

                    print_rn!("KEY :: {:?}\r", k);
                }
                Err(e) => eprint_rn!("Error: {:?}", e),
            }

            stdout.flush().unwrap();
        }
    });
}
