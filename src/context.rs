use regex::Regex;
use std::sync::Arc;
use shared_str::ArcStr;
use failure::Error;
use crate::Message;

#[derive(Clone)]
pub struct Context {
    body: ArcStr,
    message: Arc<Message>,
}

impl Context {
    pub fn new(message: Arc<Message>) -> Option<Self> {
        Some(Self {
            body: message.body(),
            message,
        })
    }

    pub fn body(&self) -> ArcStr {
        self.body.clone()
    }

    /// Wether the message was aimed directetly at the bot,
    /// either via private message or by prefixing a channel message with
    /// the bot's name, followed by ',' or ':'.
    pub fn is_directly_addressed(&self) -> bool {
        self.message.is_directly_addressed()
    }

    pub fn reply<S: AsRef<str>>(&self, message: S) -> Result<(), Error> {
        self.message.reply(message.as_ref())
    }

    pub fn source_nickname(&self) -> ArcStr {
        self.message.source_nickname()
    }

    pub fn current_nickname(&self) -> ArcStr {
        self.message.current_nickname()
    }

    pub fn inline_contexts(&self) -> Vec<Context> {
        lazy_static! {
            static ref INLINE_CMD: Regex = Regex::new(r"\{(.*?)}").unwrap();
        }

        let body = if self.is_directly_addressed() { "".into() } else { self.body.clone() };

        let contexts = INLINE_CMD
            .captures_iter(&body)
            .flat_map(|caps| caps.get(1))
            .map(|inline_body| Context {
                body: body.sliced(inline_body.as_str()).unwrap(),
                message: self.message.clone(),
            })
            .collect();
        
        contexts
    }
}
