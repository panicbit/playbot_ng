use failure::Error;
use shared_str::ArcStr;

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
}
