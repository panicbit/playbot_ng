#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate slog;

use std::thread;
use std::sync::Arc;
use actix::prelude::*;
use slog::Logger;

mod message;
pub use self::message::Message;

pub mod modules_ng;
use modules_ng::{PluginManager, event::OnMessage};

pub struct Playbot {
    plugin_manager: Addr<PluginManager>,
}

impl Playbot {
    pub fn new() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        thread::spawn(move || {
            let system = System::new("bot");

            let plugin_manager = PluginManager::create(|ctx| {
                let mut pm = PluginManager::new(&ctx);

                pm.register_plugin("help", |ctx| modules_ng::Help::new(ctx));
                pm.register_plugin("playground", |ctx| modules_ng::Playground::new(ctx));
                pm.register_plugin("crate_info", |ctx| modules_ng::CrateInfo::new(ctx));
                pm.register_plugin("egg", |ctx| modules_ng::Egg::new(ctx));
                pm.register_plugin("genword", |ctx| modules_ng::GenWord::new(ctx));

                pm
            });

            tx.send(plugin_manager);

            system.run();
        });

        let plugin_manager = rx.recv().unwrap();

        Self {
            plugin_manager,
        }
    }

    pub fn handle_message<M: Message + 'static>(&self, message: M, l: &Logger) {
        let l = l.new(o!{
            "body" => message.body().to_string(),
            "sender" => message.source_nickname().to_string(),
            "directly_addressed" => message.is_directly_addressed(),
        });

        info!(l, "Handling message");

        // self.commands.clone().handle_message(&message);
        let message = Arc::new(message) as Arc<Message>;
        let inline_messages = message.inline_messages(&message);

        if inline_messages.len() == 0 {
            self.plugin_manager.do_send(OnMessage { message, l });
        } else {
            for message in inline_messages {
                let l = l.new(o!{ "inline_body" => message.body().to_string() });
                info!(l, "Handling inline message");
                self.plugin_manager.do_send(OnMessage { message, l });
            }
        }
    }
}
