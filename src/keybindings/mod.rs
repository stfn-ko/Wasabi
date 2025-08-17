pub(crate) use termion::event::Key;
pub(crate) use tungstenite::Message;

use std::collections::HashMap;

pub struct Keybindings {
    internal: HashMap<Key, fn() -> Message>,
}

impl Keybindings {
    pub fn new() -> Self {
        Self {
            internal: HashMap::new(),
        }
    }

    pub fn add(&mut self, key: Key, message: fn() -> Message) {
        if self.internal.contains_key(&key) {
            panic!("Keybinding with value `{:?}` already exists", key);
        }

        self.internal.insert(key, message);
    }
    
    pub fn at(&self, key: Key) -> Option<&fn() -> Message> {
        self.internal.get(&key)
    }
}
