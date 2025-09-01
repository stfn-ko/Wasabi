use ::wasabi;
use wasabi::{Key, Message};

#[tokio::main]
async fn main() {
    let settings = wasabi::WebSocketSettings::builder()
        .on_connect_message(wasabi::Message::from("welcome"))
        .add_keybinding(Key::Char('t'), || Message::from("server test message"))
        .log_incoming_messages()
        .build()
        .expect("Failed to build WebSocketSettings");

    wasabi::Server::builder()
        .address("127.0.0.1:8080")
        .settings(settings)
        .build()
        .unwrap()
        .start()
        .await;
}