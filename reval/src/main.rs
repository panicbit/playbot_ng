use playbot::{Playbot, Message};
use failure::Error;
use shared_str::ArcStr;
use rustyline::error::ReadlineError;
use slog::{Logger, Discard, o};

struct CliMessage {
    body: ArcStr,
}

impl CliMessage {
    fn new(body: impl AsRef<str>) -> Self {
        Self {
            body: body.as_ref().trim().into(),
        }
    }
}

impl Message for CliMessage {
    fn body(&self) -> ArcStr {
        self.body.clone()
    }

    fn is_directly_addressed(&self) -> bool {
        true
    }

    fn reply(&self, message: &str) -> Result<(), Error> {
        println!("{}", message);
        Ok(())
    }

    fn source_nickname(&self) -> ArcStr {
        "".into()
    }

    fn current_nickname(&self) -> ArcStr {
        "".into()
    }
}

#[tokio::main]
async fn main() {
    let logger = Logger::root(Discard, o!());
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
        playbot.handle_message(message, &logger).await;
    }
}
