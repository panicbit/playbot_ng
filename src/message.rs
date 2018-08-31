use failure::Error;
use std::sync::Arc;
use irc::client::prelude::{IrcClient, ChannelExt, ClientExt, Command};

pub trait Message<'a> {
    /// The body of the message without address prefixes.
    /// E.g. `bot: hello` would be returned as `hello`.
    fn body(&self) -> &'a str;

    /// Wether the message was aimed directetly at the bot,
    /// either via private message or by prefixing a channel message with
    /// the bot's name, followed by ',' or ':'.
    fn is_directly_addressed(&self) -> bool;

    fn reply<S: AsRef<str>>(&self, message: S) -> Result<(), Error>;

    fn source_nickname(&self) -> &'a str;

    fn current_nickname(&self) -> Arc<String>;
}

type SendFn = fn(&IrcClient, &str, &str) -> irc::error::Result<()>;

#[derive(Clone)]
pub struct IrcMessage<'a> {
    body: &'a str,
    is_directly_addressed: bool,
    reply_fn: SendFn,
    source: &'a str,
    source_nickname: &'a str,
    target: &'a str,
    client: &'a IrcClient,
    current_nickname: Arc<String>,
}

impl<'a> IrcMessage<'a> {
    pub fn new(client: &'a IrcClient, message: &'a irc::proto::Message) -> Option<Self> {
        let mut body = match message.command {
            Command::PRIVMSG(_, ref body) => body.trim(),
            _ => return None,
        };

        let current_nickname = Arc::new(client.current_nickname().to_owned());

        let source_nickname = message.source_nickname()?;

        // Check wether message is ctcp
        {
            let is_ctcp = body.len() >= 2 && body.chars().next() == Some('\x01')
                && body.chars().last() == Some('\x01');

            if is_ctcp {
                return None;
            }
        }

        let source = message.prefix.as_ref().map(<_>::as_ref)?;

        let target = match message.response_target() {
            Some(target) => target,
            None => {
                eprintln!("Unknown response target");
                return None;
            }
        };

        let is_directly_addressed = {
            if body.starts_with(current_nickname.as_str()) {
                let new_body = body[current_nickname.len()..].trim_left();
                let has_separator = new_body.starts_with(":") || new_body.starts_with(",");

                if has_separator {
                    body = new_body[1..].trim_left();
                }

                has_separator
            } else {
                !target.is_channel_name()
            }
        };

        let reply_fn: SendFn = match target.is_channel_name() {
            true => |client, target, message| client.send_notice(target, message),
            false => |client, target, message| client.send_privmsg(target, message),
        };

        Some(Self {
            client,
            body,
            reply_fn,
            source,
            source_nickname,
            target,
            is_directly_addressed,
            current_nickname
        })
    }
}

impl<'a> Message<'a> for IrcMessage<'a> {
    fn body(&self) -> &'a str {
        self.body
    }

    /// Wether the message was aimed directetly at the bot,
    /// either via private message or by prefixing a channel message with
    /// the bot's name, followed by ',' or ':'.
    fn is_directly_addressed(&self) -> bool {
        self.is_directly_addressed
    }

    fn reply<S: AsRef<str>>(&self, message: S) -> Result<(), Error> {
        let message = message.as_ref();
        eprintln!("Replying: {:?}", message);
        for line in message.lines() {
            if line.len() > 400 {
                (self.reply_fn)(self.client, self.target, "<<<message too long for irc>>>")?;
                continue;
            }
            (self.reply_fn)(self.client, self.target, line)?;
        }

        Ok(())
    }

    fn source_nickname(&self) -> &'a str {
        self.source_nickname
    }

    fn current_nickname(&self) -> Arc<String> {
        self.current_nickname.clone()
    }
}
