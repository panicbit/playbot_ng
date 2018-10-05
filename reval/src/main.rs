use playbot::{Playbot, Message};
use failure::Error;
use std::sync::Arc;
use futures::prelude::*;
use futures::compat::TokioDefaultSpawner;
use rustyline::error::ReadlineError;

struct CliMessage {
    body: String,
}

impl CliMessage {
    fn new(body: String) -> Self {
        Self { body }
    }
}

impl Message for CliMessage {
    fn body(&self) -> &str {
        self.body.trim()
    }

    fn is_directly_addressed(&self) -> bool {
        true
    }

    fn reply(&self, message: &str) -> Result<(), Error> {
        println!("{}", message);
        Ok(())
    }

    fn source_nickname(&self) -> &str {
        ""
    }

    fn current_nickname(&self) -> Arc<String> {
        Arc::new(String::new())
    }
}

fn main() {
    let playbot = Playbot::new();
    let mut rl = rustyline::Editor::<()>::new();

    loop {
        let input = match rl.readline("> ") {
            Ok(input) => input,
            Err(ReadlineError::Utf8Error) |
            Err(ReadlineError::Eof) |
            Err(ReadlineError::Interrupted) => break,
            Err(err) => {
                println!("{}", err);
                break;
            }
        };
        rl.add_history_entry(input.as_str());

        let message = CliMessage::new(input);
        let fut = playbot.handle_message(message).boxed().compat(TokioDefaultSpawner);

        if let Err(e) = tokio::runtime::current_thread::block_on_all(fut) {
            println!("ERROR: {}", e);
        }
    }
}
