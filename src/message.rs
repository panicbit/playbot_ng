use failure::Error;
use std::sync::Arc;
use shared_str::ArcStr;
use regex::Regex;

pub trait Message: Send + Sync {
    /// The body of the message without address prefixes.
    /// E.g. `bot: hello` would be returned as `hello`.
    fn body(&self) -> ArcStr;

    /// Wether the message was aimed directetly at the bot,
    /// either via private message or by prefixing a channel message with
    /// the bot's name, followed by ',' or ':'.
    fn is_directly_addressed(&self) -> bool;

    fn reply(&self, message: &str) -> Result<(), Error>;

    fn source_nickname(&self) -> ArcStr;

    fn current_nickname(&self) -> ArcStr;

    fn inline_messages(&self, message: &Arc<Message>) -> Vec<Arc<Message>> {
        InlineMessage::from_message(message)
    }
}

#[derive(Clone)]
pub struct InlineMessage {
    body: ArcStr,
    message: Arc<Message>,
}

impl InlineMessage {
    pub fn from_message(message: &Arc<Message>) -> Vec<Arc<Message>> {
        lazy_static! {
            static ref INLINE_CMD: Regex = Regex::new(r"\{(.*?)}").unwrap();
        }

        let body = if message.is_directly_addressed() { "".into() } else { message.body() };

        let messages = INLINE_CMD
            .captures_iter(&body)
            .flat_map(|caps| caps.get(1))
            .map(|inline_body| Self {
                body: body.sliced(inline_body.as_str()).unwrap(),
                message: message.clone(),
            })
            .map(|m| Arc::new(m) as Arc<Message>)
            .collect();

        messages
    }
}

impl Message for InlineMessage {
    fn body(&self) -> ArcStr {
        self.body.clone()
    }

    fn is_directly_addressed(&self) -> bool {
        self.message.is_directly_addressed()
    }

    fn reply(&self, message: &str) -> Result<(), Error> {
        self.message.reply(message)
    }

    fn source_nickname(&self) -> ArcStr {
        self.message.source_nickname()
    }

    fn current_nickname(&self) -> ArcStr {
        self.message.current_nickname()
    }
}
