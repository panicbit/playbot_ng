#![feature(box_patterns)]
#![feature(futures_api)]
#![feature(async_await)]
#![feature(await_macro)]
#![feature(arbitrary_self_types)]
extern crate failure;
extern crate irc;
extern crate reqwest;
extern crate url;
extern crate chrono;
extern crate itertools;
extern crate regex;
extern crate playground;
extern crate cratesio;
extern crate serde;
extern crate toml;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_derive;
extern crate futures;
extern crate tokio_core;

use std::thread;
use std::sync::Arc;
use chrono::{
    prelude::*,
    Duration,
};
use irc::client::prelude::{Config as IrcConfig, IrcReactor, ClientExt};
use futures::prelude::*;
use futures::compat::TokioDefaultSpawn;
use failure::Error;
use self::{
    context::Context,
    command::Command,
    command_registry::CommandRegistry,
};
use crate::module::Module;
pub use self::config::Config;
use self::message::IrcMessage;
use tokio_core::reactor::Handle;

mod context;
mod command;
mod command_registry;
mod module;
// mod codedb;
mod config;
mod message;

pub struct Playbot {
    commands: Arc<CommandRegistry>,
    handle: Handle,
}

impl Playbot {
    pub fn new(handle: Handle) -> Self {
        let mut commands = CommandRegistry::new("?");

        module::CrateInfo::init(&mut commands);
        module::Help::init(&mut commands);
        module::Egg::init(&mut commands);
        module::Playground::init(&mut commands);

        Self {
            commands: Arc::new(commands),
            handle,
        }
    }

    pub fn handle_message<'a>(&self, message: IrcMessage<'a>) -> impl Future<Output = Result<(), Error>> + 'a {
        let commands = self.commands.clone();
        let handle = self.handle.clone();

        async move {
            await!(commands.handle_message(handle, &message))
        }
    }
}

pub fn main() {
    let config = Config::load("config.toml").expect("failed to load config.toml");

    let threads: Vec<_> = config.instances.into_iter().map(|config| {
        thread::spawn(move || run_instance(config))
    }).collect();

    for thread in threads {
        thread.join().ok();
    }
}

pub fn run_instance(config: IrcConfig) {
    let sleep_dur = Duration::seconds(5).to_std().unwrap();

    loop {   
        println!("{} Starting up", Utc::now());

        match connect_and_handle(config.clone()) {
            Ok(()) => eprintln!("[OK] Disconnected for an unknown reason"),
            Err(e) => {
                eprintln!("[ERR] Disconnected");

                for cause in e.iter_chain() {
                    eprintln!("[CAUSE] {}", cause);
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
    let handle = reactor.inner_handle().clone();
    let client = reactor.prepare_client_and_connect(config)?;
    let playbot = Arc::new(Playbot::new(handle.clone()));

    client.identify()?;

    reactor
    .register_client_with_handler(client, move |client, message| {
        let playbot = playbot.clone();
        let client = client.clone();

        handle.spawn({
            async move {
                let message = match IrcMessage::new(&client, &message) {
                    Some(message) => message,
                    None => return Ok(()),
                };

                match await!(playbot.handle_message(message)) {
                    Ok(_) => {},
                    Err(e) => {
                        eprintln!("[ERR] {}", e);

                        for cause in e.iter_chain() {
                            eprintln!("[CAUSE]: {}", cause);
                        }
                    },
                };

                Ok(())
            }
        }.boxed().compat(TokioDefaultSpawn));

        Ok(())
    });

    // reactor blocks until a disconnection or other in `irc` error
    reactor.run()?;

    Ok(())
}

#[derive(PartialEq, Eq)]
pub enum Flow {
    Break,
    Continue,
}
