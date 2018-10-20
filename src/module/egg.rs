use crate::module::prelude::*;
use regex::Regex;
use itertools::Itertools;
use std::iter::once;
use futures::prelude::*;

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
        )
    ];
}

pub(crate) enum Egg {}

impl Module for Egg {
    fn init(commands: &mut CommandRegistry) {
        commands.add_fallback_handler(egg_handler);
    }
}

fn egg_handler<'a>(ctx: &'a Context) {
    for dialog in &*SCRIPT {
        if let Some(caps) = dialog.0.captures(ctx.body()) {
            if let Some(nick) = caps.name("nick") {
                if nick.as_str() != ctx.current_nickname().as_str() {
                    return;
                }
            }

            let reply = (dialog.1)(ctx.source_nickname());

            if !reply.is_empty() {
                ctx.reply(&reply);
            }

            return;
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
