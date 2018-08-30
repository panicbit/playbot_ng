use irc;
use irc::client::prelude::*;
use regex::Regex;
use std::sync::Arc;
use failure::Error;

type SendFn = fn(&IrcClient, &str, &str) -> irc::error::Result<()>;

#[derive(Clone)]
pub struct Context<'a> {
    body: &'a str,
    message: IrcMessage<'a>,
}

#[derive(Clone)]
struct IrcMessage<'a> {
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
    pub fn new(client: &'a IrcClient, message: &'a Message) -> Option<Self> {
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

    pub fn body(&self) -> &'a str {
        self.body
    }

    /// Wether the message was aimed directetly at the bot,
    /// either via private message or by prefixing a channel message with
    /// the bot's name, followed by ',' or ':'.
    pub fn is_directly_addressed(&self) -> bool {
        self.is_directly_addressed
    }

    pub fn reply<S: AsRef<str>>(&self, message: S) -> Result<(), Error> {
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

    pub fn source_nickname(&self) -> &'a str {
        self.source_nickname
    }

    pub fn current_nickname(&self) -> &Arc<String> {
        &self.current_nickname
    }
}

impl<'a> Context<'a> {
    pub fn new(client: &'a IrcClient, message: &'a Message) -> Option<Self> {
        let message = IrcMessage::new(client, message)?;

        Some(Self {
            body: message.body(),
            message,
        })
    }

    pub fn body(&self) -> &'a str {
        self.body
    }

    /// Wether the message was aimed directetly at the bot,
    /// either via private message or by prefixing a channel message with
    /// the bot's name, followed by ',' or ':'.
    pub fn is_directly_addressed(&self) -> bool {
        self.message.is_directly_addressed()
    }

    pub fn reply<S: AsRef<str>>(&self, message: S) -> Result<(), Error> {
        self.message.reply(message)
    }

    pub fn source_nickname(&self) -> &'a str {
        self.message.source_nickname()
    }

    pub fn current_nickname(&self) -> &Arc<String> {
        self.message.current_nickname()
    }

    pub fn inline_contexts<'b>(&'b self) -> impl Iterator<Item = Context<'a>> + 'b {
        lazy_static! {
            static ref INLINE_CMD: Regex = Regex::new(r"\{(.*?)}").unwrap();
        }

        let body = if self.is_directly_addressed() { "" } else { self.body };

        let contexts = INLINE_CMD
            .captures_iter(body)
            .flat_map(|caps| caps.get(1))
            .map(move |body| Context {
                body: body.as_str(),
                .. self.clone()
            });
        
        Box::new(contexts)
    }
}
