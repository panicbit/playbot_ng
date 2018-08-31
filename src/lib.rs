#![feature(box_patterns)]
#![feature(futures_api)]
#![feature(async_await)]
#![feature(await_macro)]
#![feature(arbitrary_self_types)]
#[macro_use] extern crate lazy_static;

use std::sync::Arc;
use futures::prelude::*;
use failure::Error;
use self::{
    context::Context,
    command::Command,
    command_registry::CommandRegistry,
};
use crate::module::Module;

mod context;
mod command;
mod command_registry;
mod module;
// mod codedb;
mod message;
pub use self::message::Message;

pub struct Playbot {
    commands: Arc<CommandRegistry>,
}

impl Playbot {
    pub fn new() -> Self {
        let mut commands = CommandRegistry::new("?");

        module::CrateInfo::init(&mut commands);
        module::Help::init(&mut commands);
        module::Egg::init(&mut commands);
        module::Playground::init(&mut commands);

        Self {
            commands: Arc::new(commands),
        }
    }

    pub fn handle_message<'a, M: Message + 'a>(&self, message: M) -> impl Future<Output = Result<(), Error>> + 'a {
        let commands = self.commands.clone();

        async move {
            await!(commands.handle_message(&message))
        }
    }
}

#[derive(PartialEq, Eq)]
pub(crate) enum Flow {
    Break,
    Continue,
}
