use actix::prelude::*;
use crate::Message;
use super::{PluginContext, OnCommand};

pub struct Help {}

impl Help {
    pub fn new(ctx: PluginContext<Self>) -> Self {
        ctx.on_command("help", ctx.recipient());
        Self {}
    }
}

impl Actor for Help {
    type Context = Context<Self>;
}

impl Handler<OnCommand> for Help {
    type Result = ();

    fn handle(&mut self, event: OnCommand, ctx: &mut Context<Self>) {
        if event.command != "help" {
            return;
        }

        display_help(&*event.message);
    }
}

pub(crate) fn display_help(ctx: &Message) {
    ctx.reply("Usage help can be found here: https://github.com/panicbit/playbot_ng/tree/master/README.md");
}
