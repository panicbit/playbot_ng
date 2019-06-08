use actix::prelude::*;
use crate::Message;
use super::{PluginContext, OnMessage, Priority};
use regex::Regex;
use itertools::Itertools;
use std::iter::once;

lazy_static! {
    static ref SCRIPT: Vec<(Regex, fn(&str) -> String)> = vec![
        (
            re(r"Open the [[:word:]]( [[:word:]]+)* bay doors? ,? (?P<nick>[[:word:]]+) [.!]?"),
            |name| format!("I'm sorry {}, I'm afraid I can't do that.", name),
        ),
        (
            re(r"(What'?s|What is|Wats) the problem \??"),
            |_| format!("I think you know what the problem is just as well as I do."),
        ),
        (
            re(r"What are you talking about ,? (?P<nick>[[:word:]]+) \??"),
            |_| format!("This mission is too important for me to allow you to jeopardize it.")
        ),
        (
            re(r"I (don't|dont) know what you are talking about ,? (?P<nick>[[:word:]]+) [.?!]?"),
            |name| {
                let other = match name.to_lowercase().as_str() {
                    "panicbit" => "Rantanen",
                    "rantanen" => "panicbit",
                    "graydon" => "steveklabnik",
                    _ => "Graydon",
                };
                format!("I know that you and {} were planning to disconnect me and I'm afraid that's something I cannot allow to happen", other)
            }
        ),
        (
            re(r"(You're|You are) doing good work,? (?P<nick>[[:word:]]+)!?"),
            |name| match name {
                "rustbot" | "[o__o]" => format!("Thank you {}!", name),
                _ => String::new(),
            }
        ),
        (
            re(r"\?hey (?P<nick>[[:word:]]+)"),
            |name| match name {
                "rustbot" => format!("hey j and rink"),
                _ => String::new(),
            }
        ),
        (
            re(r"\?botchain"),
            |_| format!("hey j and rink"),
        )
    ];
}

pub struct Egg {}

impl Egg {
    pub fn new(ctx: PluginContext<Self>) -> Self {
        ctx.on_message(Priority::NORMAL, ctx.recipient());
        Self {}
    }
}

impl Actor for Egg {
    type Context = Context<Self>;
}

impl Handler<OnMessage> for Egg {
    type Result = ();

    fn handle(&mut self, event: OnMessage, ctx: &mut Context<Self>) {
        let message = event.message;

        for dialog in &*SCRIPT {
            if let Some(caps) = dialog.0.captures(&message.body()) {
                if let Some(nick) = caps.name("nick") {
                    if nick.as_str() != message.current_nickname().as_str() {
                        return;
                    }
                }

                let reply = (dialog.1)(&message.source_nickname());

                if !reply.is_empty() {
                    message.reply(&reply);
                }

                return;
            }
        }
    }
}

fn re(re: &str) -> Regex {
    let re = once("(?i)^")
        .chain(re.split_whitespace())
        .chain(once("$"))
        .join(r"\s*");
    Regex::new(&re).unwrap()
}
