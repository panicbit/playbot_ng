use actix::prelude::*;
use crate::Message;
use super::{PluginContext, OnCommand};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use rand::{thread_rng, seq::SliceRandom};

pub struct GenWord {}

impl GenWord {
    pub fn new(ctx: PluginContext<Self>) -> Self {
        ctx.on_command("genword", ctx.recipient());
        Self {}
    }
}

impl Actor for GenWord {
    type Context = Context<Self>;
}

impl Handler<OnCommand> for GenWord {
    type Result = ();

    fn handle(&mut self, event: OnCommand, ctx: &mut Context<Self>) {
        if event.command != "genword" {
            return;
        }

        let message = event.message;

        match gen_word() {
            Ok(word) => {
                message.reply(&word);
            },
            Err(e) => {
                message.reply("Failed to generate word");
                eprintln!("[genword] {}", e);
            }
        };
    }
}

lazy_static! {
    static ref WORDS: Result<Vec<String>, Box<Error + Send + Sync>> = {
        let file = File::open("/usr/share/dict/american-english")?;
        let file = BufReader::new(file);
        let mut words = Vec::new();

        for word in file.lines() {
            let word = word?;
            if !word.chars().all(|c| c.is_lowercase() && c.is_alphanumeric()) {
                continue
            }

            if word.len() < 3 {
                continue;
            }

            words.push(word);
        }

        Ok(words)
    };
}

fn gen_word() -> Result<String, String> {
    let word = format!("{}{}", random_word()?, random_word()?);
    Ok(word)
}

fn random_word() -> Result<&'static str, String> {
    let words = WORDS.as_ref().map_err(<_>::to_string)?;
    let word = words.choose(&mut thread_rng()).ok_or("No word in word list")?;
    Ok(&*word)
}
