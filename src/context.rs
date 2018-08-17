use irc;
use irc::client::prelude::*;
use regex::Regex;
use std::sync::Arc;

type SendFn = fn(&IrcClient, &str, &str) -> irc::error::Result<()>;

#[derive(Clone)]
pub struct Context<'a> {
    body: &'a str,
    is_directly_addressed: bool,
    is_ctcp: bool,
    send_fn: SendFn,
    source: &'a str,
    source_nickname: &'a str,
    target: &'a str,
    client: &'a IrcClient,
    current_nickname: Arc<String>,
}

impl<'a> Context<'a> {
    pub fn new(client: &'a IrcClient, message: &'a Message) -> Option<Self> {
        let mut body = match message.command {
            Command::PRIVMSG(_, ref body) => body.trim(),
            _ => return None,
        };

        let current_nickname = Arc::new(client.current_nickname().to_owned());

        let source_nickname = message.source_nickname()?;

        let is_ctcp = body.len() >= 2 && body.chars().next() == Some('\x01')
            && body.chars().last() == Some('\x01');

        if is_ctcp {
            body = &body[1..body.len() - 1];
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

        let send_fn: SendFn = match target.is_channel_name() {
            true => |client, target, message| client.send_notice(target, message),
            false => |client, target, message| client.send_privmsg(target, message),
        };

        Some(Self {
            client,
            body,
            send_fn,
            source,
            source_nickname,
            target,
            is_directly_addressed,
            is_ctcp,
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

    pub fn is_ctcp(&self) -> bool {
        self.is_ctcp
    }

    pub fn reply<S: AsRef<str>>(&self, message: S) {
        let message = message.as_ref();
        eprintln!("Replying: {:?}", message);
        for line in message.lines() {
            if line.len() > 400 {
                (self.send_fn)(self.client, self.target, "<<<message too long for irc>>>");
                continue;
            }
            (self.send_fn)(self.client, self.target, line);
        }
    }

    pub fn source(&self) -> &'a str {
        self.source
    }

    pub fn source_nickname(&self) -> &'a str {
        self.source_nickname
    }

    pub fn current_nickname(&self) -> Arc<String> {
        self.current_nickname.clone()
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
