use crate::module::prelude::*;
use crate::Command;

pub(crate) enum Help {}

impl Module for Help {
    fn init(commands: &mut CommandRegistry) {
        commands.set_named_handler("help", help_handler);
    }
}

fn help_handler(ctx: &Context, _args: &Command) {
    display_help(ctx);
}

pub(crate) fn display_help(ctx: &Context) {
    ctx.reply("Usage help can be found here: https://github.com/panicbit/playbot_ng/tree/master/README.md");
}
