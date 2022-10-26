use aldrin_proto::Message;
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::{env, fmt};

pub struct Messages {
    messages: HashMap<MessageSize, Message>,
}

impl Messages {
    pub fn open() -> Self {
        let mut messages = HashMap::new();

        messages.insert(MessageSize::Small, Self::parse_message("small.json"));
        messages.insert(MessageSize::Large, Self::parse_message("large.json"));

        Messages { messages }
    }

    fn parse_message(name: impl AsRef<Path>) -> Message {
        let mut path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        path.extend(["benches", "datasets", "messages"]);
        path.push(name.as_ref());
        let file = File::open(path).unwrap();
        serde_json::from_reader(file).unwrap()
    }

    pub fn get(&self, size: MessageSize) -> &Message {
        self.messages.get(&size).unwrap()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum MessageSize {
    Small,
    Large,
}

impl fmt::Display for MessageSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let size = match self {
            MessageSize::Small => "small",
            MessageSize::Large => "large",
        };
        f.write_str(size)
    }
}
