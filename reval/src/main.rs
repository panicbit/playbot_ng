use playbot::{Playbot, Message};
use failure::Error;
use std::sync::Arc;
use tokio_core::reactor::{Core, Handle};
use futures::prelude::*;
use futures::compat::TokioDefaultSpawn;
use std::io::{self, Write};

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
    let mut reactor = Core::new().unwrap();
    let handle = reactor.handle().clone();
    let playbot = Playbot::new(handle);
    let stdout = io::stdout();
    let stdin = io::stdin();

    loop {
        print!("> ");
        stdout.lock().flush().unwrap();
        let mut input = String::new();

        if stdin.read_line(&mut input).unwrap() == 0 {
            return;
        }

        let message = CliMessage::new(input);
        let fut = playbot.handle_message(message).boxed().compat(TokioDefaultSpawn);
        reactor.run(fut).unwrap();
    }
}
