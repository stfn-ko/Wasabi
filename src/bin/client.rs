use ::wasabi;
use wasabi::{Key, Message};

#[tokio::main]
async fn main() {
    let settings = wasabi::WebSocketSettings::builder()
        .on_connect_message(wasabi::Message::from("hello server"))
        .add_keybinding(Key::Char('t'), || Message::from("client test message"))
        .log_incoming_messages()
        .build()
        .expect("Failed to build WebSocketSettings");

    wasabi::Client::builder()
        .address("ws://127.0.0.1:8080")
        .settings(settings)
        .build()
        .expect("Couldn't build Client")
        .start()
        .await;
}
