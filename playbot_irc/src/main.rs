#[macro_use] extern crate serde_derive;

use std::thread;
use std::sync::Arc;
use shared_str::ArcStr;
use chrono::{
    prelude::*,
    Duration,
};
use irc::client::prelude::{*, Config as IrcConfig};
use failure::Error;
use playbot::{Playbot, Message};
use std::panic::catch_unwind;

mod config;
use self::config::Config;

pub fn main() {
    let config = Config::load("config.toml").expect("failed to load config.toml");

    let threads: Vec<_> = config.instances.into_iter().map(|config| {
        thread::spawn(move || loop {
            if catch_unwind(|| run_instance(config.clone())).is_err() {
                println!("PANICKED");
            }
        })
    }).collect();

    for thread in threads {
        thread.join().ok();
    }
}

pub fn run_instance(config: IrcConfig) {
    let sleep_dur = Duration::seconds(5).to_std().unwrap();
    let server = config.server.as_ref().map(|x| &**x).unwrap_or("");

    loop {   
        println!("{} Starting up", Utc::now());

        match connect_and_handle(config.clone()) {
            Ok(()) => eprintln!("{}/[OK] Disconnected for an unknown reason", server),
            Err(e) => {
                eprintln!("[{}/ERR] Disconnected", server);

                for cause in e.iter_chain() {
                    eprintln!("[{}/CAUSE] {}", server, cause);
                }
            }
        }

        eprintln!("Reconnecting in 5 seconds");

        thread::sleep(sleep_dur);

        println!("{} Terminated", Utc::now());
    }
}

pub fn connect_and_handle(config: IrcConfig) -> Result<(), Error> {
    //    let mut codedb = ::codedb::CodeDB::open_or_create("code_db.json")?;
    let mut reactor = IrcReactor::new()?;
    let client = reactor.prepare_client_and_connect(config)?;
    let playbot = Arc::new(Playbot::new());

    client.identify()?;

    reactor
    .register_client_with_handler(client, move |client, message| {
        let playbot = playbot.clone();
        let client = client.clone();

        let message = match IrcMessage::new(client, message) {
            Some(message) => message,
            None => return Ok(()),
        };

        playbot.handle_message(message);

        Ok(())
    });

    // reactor blocks until a disconnection or other in `irc` error
    reactor.run()?;

    Ok(())
}

type SendFn = fn(&IrcClient, &str, &str) -> irc::error::Result<()>;

#[derive(Clone)]
pub struct IrcMessage {
    body: ArcStr,
    is_directly_addressed: bool,
    reply_fn: SendFn,
    source: Prefix,
    source_nickname: ArcStr,
    target: ArcStr,
    client: IrcClient,
    current_nickname: ArcStr,
}

impl IrcMessage {
    pub fn new(client: IrcClient, message: irc::proto::Message) -> Option<Self> {
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

        let source = message.prefix.as_ref()?;

        let target = match message.response_target() {
            Some(target) => target,
            None => {
                eprintln!("Unknown response target");
                return None;
            }
        };

        let is_directly_addressed = {
            if body.starts_with(current_nickname.as_str()) {
                let new_body = body[current_nickname.len()..].trim_start();
                let has_separator = new_body.starts_with(":") || new_body.starts_with(",");

                if has_separator {
                    body = new_body[1..].trim_start();
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
            body: body.into(),
            reply_fn,
            source: source.to_owned(),
            source_nickname: source_nickname.into(),
            target: target.into(),
            is_directly_addressed,
            current_nickname: current_nickname.to_string().into(),
        })
    }
}

impl Message for IrcMessage {
    fn body(&self) -> ArcStr {
        self.body.clone()
    }

    /// Wether the message was aimed directetly at the bot,
    /// either via private message or by prefixing a channel message with
    /// the bot's name, followed by ',' or ':'.
    fn is_directly_addressed(&self) -> bool {
        self.is_directly_addressed
    }

    fn reply(&self, message: &str) -> Result<(), Error> {
        eprintln!("Replying: {:?}", message);
        for line in message.lines().flat_map(|line| line.split('\r')) {
            if line.len() > 400 {
                (self.reply_fn)(&self.client, &self.target, "<<<message too long for irc>>>")?;
                continue;
            }
            (self.reply_fn)(&self.client, &self.target, line)?;
        }

        Ok(())
    }

    fn source_nickname(&self) -> ArcStr {
        self.source_nickname.clone()
    }

    fn current_nickname(&self) -> ArcStr {
        self.current_nickname.clone()
    }
}
