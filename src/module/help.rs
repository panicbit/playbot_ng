use crate::module::prelude::*;

pub(crate) enum Help {}

impl Module for Help {
    fn init(commands: &mut CommandRegistry) {
        commands.set_named_handler("help", help_handler);
    }
}

fn help_handler(ctx: &Context, _args: &[&str]) -> Flow {
    display_help(ctx);
    Flow::Break
}

pub(crate) fn display_help(ctx: &Context) {
    ctx.reply("Usage help can be found here: https://github.com/panicbit/playbot_ng/tree/master/README.md");
}
