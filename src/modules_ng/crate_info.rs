use actix::prelude::*;
use crate::Message;
use super::{PluginContext, OnCommand};
use cratesio;
use url::percent_encoding::{utf8_percent_encode, PATH_SEGMENT_ENCODE_SET};
use itertools::Itertools;
use reqwest::StatusCode;


pub struct CrateInfo {}

impl CrateInfo {
    pub fn new(ctx: PluginContext<Self>) -> Self {
        ctx.on_command("crate", ctx.recipient());
        Self {}
    }
}

impl Actor for CrateInfo {
    type Context = Context<Self>;
}

impl Handler<OnCommand> for CrateInfo {
    type Result = ();

    fn handle(&mut self, event: OnCommand, ctx: &mut Context<Self>) {
        if event.command != "crate" {
            return;
        }

        for crate_name in event.arg.split_whitespace().take(3) {
            show_crate_info(&*event.message, crate_name);
        }
    }
}

fn show_crate_info(ctx: &Message, crate_name: &str) {
    let info = match cratesio::crate_info(crate_name) {
        Ok(info) => info,
        // TODO: Use proper error types
        Err(ref err) if err.status() == Some(StatusCode::NOT_FOUND) => {
            ctx.reply(&format!("Crate '{}' does not exist.", crate_name));
            return
        },
        Err(err) => {
            eprintln!("Error getting crate info for '{}': {:?}", crate_name, err);
            ctx.reply(&format!("Failed to get crate info for {}", crate_name));
            return
        }
    };

    let krate = info.krate();
    let output = format!(
        "{name} ({version}) - {description} -> https://crates.io/crates/{urlname} [https://docs.rs/crate/{urlname}]",
        name = krate.name(),
        version = krate.max_version(),
        description = krate.description().split_whitespace().join(" "),
        urlname = utf8_percent_encode(&krate.name(), PATH_SEGMENT_ENCODE_SET).collect::<String>()
    );

    ctx.reply(&output);
}
