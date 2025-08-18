use crate::common::{close, ping};
use std::collections::HashMap;
pub(crate) use termion::event::Key;
pub(crate) use tungstenite::Message;

pub struct Keybindings {
    internal: HashMap<Key, fn() -> Message>,
}

impl Keybindings {
    pub fn new() -> Self {
        let mut internal = HashMap::new();

        internal.insert(Key::Char('q'), close as fn() -> Message);
        internal.insert(Key::Char('p'), ping as fn() -> Message);

        Self { internal }
    }

    pub fn add(&mut self, key: Key, message: fn() -> Message) {
        if key == Key::Char('q') {
            panic!("Use of reserved key `{:?}` (close connection)", key);
        }

        if key == Key::Char('p') {
            panic!("Use of reserved key `{:?}` (ping)", key);
        }

        if self.internal.contains_key(&key) {
            panic!("Keybinding with value `{:?}` already exists", key);
        }

        self.internal.insert(key, message);
    }

    pub fn at(&self, key: Key) -> Option<&fn() -> Message> {
        self.internal.get(&key)
    }
}
