#![feature(box_patterns)]
#![feature(futures_api)]
#![feature(async_await)]
#![feature(await_macro)]
#![feature(arbitrary_self_types)]
extern crate playbot_ng;
#[macro_use] extern crate serde_derive;

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
use playbot_ng::{Playbot, IrcMessage};

mod config;
use self::config::Config;

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
