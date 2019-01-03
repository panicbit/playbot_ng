use std::collections::HashMap;
use super::{Context, Command};
use std::iter;
use crate::Message;

pub(crate) struct CommandRegistry {
    command_prefix: String,
    named_handlers: HashMap<String, Box<Fn(&Context, &Command)>>,
    fallback_handlers: Vec<Box<Fn(&Context)>>,
}

impl CommandRegistry {
    pub fn new(command_prefix: impl Into<String>) -> Self {
        Self {
            command_prefix: command_prefix.into(),
            named_handlers: HashMap::new(),
            fallback_handlers: Vec::new(),
        }
    }

    pub fn set_named_handler(
        &mut self,
        name: impl Into<String>,
        handler: impl Fn(&Context, &Command) + 'static,
    ) {
        self.named_handlers.insert(name.into(), Box::new(handler));
    }

    pub fn add_fallback_handler(
        &mut self,
        handler: impl Fn(&Context) + 'static,
    ) {
        self.fallback_handlers.push(Box::new(handler));
    }

    pub fn handle_message(&self, message: &Message) {
        let context = match Context::new(message) {
            Some(context) => context,
            None => return,
        };

        if self.execute_commands(&context) {
            return;
        }

        if self.execute_inline_commands(&context) {
            return;
        }

        self.execute_fallback_handlers(&context);
    }

    fn execute_commands(&self, context: &Context) -> bool {
        if let Some(command) = Command::parse(&self.command_prefix, context.body()) {
            if let Some(handler) = self.named_handlers.get(command.name()) {
                handler(&context, &command);
                return true;
            }
        }

        false
    }

    fn execute_inline_commands(&self, context: &Context) -> bool {
        let mut some_command_executed = false;
        let contexts = iter::once(context.clone()).chain(context.inline_contexts());

        for context in contexts.take(3) {
            if let Some(command) = Command::parse(&self.command_prefix, context.body()) {
                if let Some(handler) = self.named_handlers.get(command.name()) {
                    handler(&context, &command);
                    some_command_executed = true;
                }
            }
        }

        some_command_executed
    }

    fn execute_fallback_handlers(&self, context: &Context) {
        for handler in &self.fallback_handlers {
            handler(&context);
        }
    }
}
