#![feature(box_patterns)]
#![feature(option_filter)]
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

use std::thread;
use chrono::{
    prelude::*,
    Duration,
};
use irc::client::prelude::{Config as IrcConfig, IrcReactor, ClientExt};
use failure::Error;
use self::{
    context::Context,
    command::Command,
    command_registry::CommandRegistry,
};
use module::Module;
use self::config::Config;

mod context;
mod command;
mod command_registry;
mod module;
// mod codedb;
mod config;

fn main() {
    let config = Config::load("config.toml").expect("failed to load config.toml");

    let threads: Vec<_> = config.instances.into_iter().map(|config| {
        thread::spawn(move || run_instance(&config))
    }).collect();

    for thread in threads {
        thread.join().ok();
    }
}

pub fn run_instance(config: &IrcConfig) {
    let sleep_dur = Duration::seconds(5).to_std().unwrap();

    loop {   
        println!("{} Starting up", Utc::now());

        match connect_and_handle(&config) {
            Ok(()) => eprintln!("[OK] Disconnected for an unknown reason"),
            Err(e) => {
                eprintln!("[ERR] Disconnected");

                for cause in e.causes() {
                    eprintln!("[ERR] Caused by: {}", cause);
                }
            }
        }

        eprintln!("Reconnecting in 5 seconds");

        thread::sleep(sleep_dur);

        println!("{} Terminated", Utc::now());
    }
}

pub fn connect_and_handle(config: &IrcConfig) -> Result<(), Error> {
    //    let mut codedb = ::codedb::CodeDB::open_or_create("code_db.json")?;
    let mut reactor = IrcReactor::new()?;
    let client = reactor.prepare_client_and_connect(&config)?;
    let mut commands = CommandRegistry::new("?");

    module::CrateInfo::init(&mut commands);
    module::Help::init(&mut commands);
    module::Egg::init(&mut commands);
    module::Playground::init(&mut commands);

    client.identify()?;

    reactor
        .register_client_with_handler(client, move |client, message| {
            commands.handle_message(&client, &message);
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
