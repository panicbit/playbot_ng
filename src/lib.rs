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
pub use self::message::IrcMessage;
use crate::module::Module;
use tokio_core::reactor::Handle;

mod context;
mod command;
mod command_registry;
mod module;
// mod codedb;
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

#[derive(PartialEq, Eq)]
pub enum Flow {
    Break,
    Continue,
}
