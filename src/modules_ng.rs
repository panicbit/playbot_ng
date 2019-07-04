use actix::prelude::*;
use std::sync::Arc;
use std::collections::HashMap;
use std::cmp::Reverse;
use crate::message::Message;
use regex::Regex;

mod playground;
pub(crate) use self::playground::Playground;

mod help;
pub(crate) use self::help::Help;

mod crate_info;
pub(crate) use self::crate_info::CrateInfo;

mod egg;
pub(crate) use self::egg::Egg;

mod genword;
pub(crate) use self::genword::GenWord;

pub struct PluginContext<P>
where P: Actor,
{
    address: Addr<P>,
    plugin_manager: Addr<PluginManager>,
    id: PluginId,
}

impl<P> PluginContext<P>
where P: Actor,
{
    fn new(address: Addr<P>, plugin_manager: Addr<PluginManager>, id: PluginId) -> Self {
        Self { address, plugin_manager, id }
    }

    pub fn address(&self) -> &Addr<P> {
        &self.address
    }

    pub fn recipient<M>(&self) -> Recipient<M>
    where
        P: Handler<M>,
        P::Context: actix::dev::ToEnvelope<P, M>,
        M: actix::Message + Send + 'static,
        M::Result: Send,
    {
        self.address().clone().recipient()
    }

    pub fn on_message(&self, priority: Priority, recipient: Recipient<OnMessage>) {
        let handler = OnMessageHandler {
            plugin_id: self.id.clone(),
            priority,
            recipient
        };
        self.plugin_manager.do_send(RegisterOnMessageHandler { handler });
    }

    pub fn on_command(&self, command: impl Into<String>, recipient: Recipient<OnCommand>) {
        let command = command.into();
        let handler = OnCommandHandler {
            plugin_id: self.id.clone(),
            recipient,
        };

        self.plugin_manager.do_send(RegisterOnCommandHandler { handler, command });
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PluginId {
    name: Arc<String>,
}

impl PluginId {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: Arc::new(name.into()),
        }
    }
}

#[derive(Clone)]
pub struct Plugin {
    id: PluginId,
}

pub struct PluginManager {
    addr: Addr<Self>,
    plugins: HashMap<PluginId, Plugin>,
    on_message_handlers: Vec<OnMessageHandler>,
    on_command_handlers: HashMap<String, OnCommandHandler>,
}

impl PluginManager {
    pub fn new(ctx: &Context<Self>) -> Self {
        Self {
            addr: ctx.address(),
            plugins: HashMap::new(),
            on_message_handlers: Vec::new(),
            on_command_handlers: HashMap::new(),
        }
    }

    pub fn register_plugin<F, A>(&mut self, name: impl Into<String>, f: F)
    where
        F: FnOnce(PluginContext<A>) -> A + 'static,
        A: Actor<Context = Context<A>>,
    {
        let id = PluginId::new(name);

        if self.plugins.contains_key(&id) {
            eprintln!("Plugin '{}' already registered", id.name);
            return;
        }

        let plugin_manager = self.addr.clone();

        A::create(move |ctx| {
            let context = PluginContext::new(ctx.address().clone(), plugin_manager, id);
            f(context)
        });
    }
}

impl Actor for PluginManager {
    type Context = Context<Self>;
}

impl<N, F, A> Handler<RegisterPlugin<N, F, A>> for PluginManager
where
    N: Into<String>,
    F: FnOnce(PluginContext<A>) -> A + 'static,
    A: Actor<Context = Context<A>>,
{
    type Result = ();

    fn handle(&mut self, event: RegisterPlugin<N, F, A>, _ctx: &mut Context<Self>) {
        self.register_plugin(event.name, event.f)
    }
}

impl Handler<RegisterOnMessageHandler> for PluginManager {
    type Result = ();

    fn handle(&mut self, event: RegisterOnMessageHandler, _ctx: &mut Context<Self>) {
        self.on_message_handlers.push(event.handler);
        self.on_message_handlers.sort_by_key(|handler| Reverse(handler.priority));
    }
}

impl Handler<RegisterOnCommandHandler> for PluginManager {
    type Result = ();

    fn handle(&mut self, event: RegisterOnCommandHandler, _ctx: &mut Context<Self>) {
        if self.on_command_handlers.contains_key(&event.command) {
            eprintln!("Command '{}' is already registered", event.command);
            return;
        }

        self.on_command_handlers.insert(event.command, event.handler);
    }
}

impl Handler<UnloadPlugin> for PluginManager {
    type Result = ();

    fn handle(&mut self, event: UnloadPlugin, _ctx: &mut Context<Self>) {
        let plugin_id = match event {
            UnloadPlugin::ById(plugin_id) => plugin_id,
        };

        let plugin = match self.plugins.remove(&plugin_id) {
            Some(plugin) => plugin,
            None => {
                eprintln!("Cannot unload plugin: plugin does not exist");
                return
            },
        };

        eprintln!("Unloading plugin '{}'", plugin.id.name);

        self.on_message_handlers.retain(|handler| handler.plugin_id != plugin_id);
        self.on_command_handlers.retain(|_, handler| handler.plugin_id != plugin_id);
    }
}

impl Handler<OnMessage> for PluginManager {
    type Result = ();

    fn handle(&mut self, event: OnMessage, _ctx: &mut Context<Self>) {
        lazy_static! {
            static ref COMMAND_RE: Regex =
                Regex::new(r"^\s*\?(?P<command>\w+)\s*(?P<arg>.*)\s*$").unwrap();
        }

        match COMMAND_RE.captures(&event.message.body()) {
            Some(captures) => {
                let command = captures["command"].to_string();
                let arg = captures["arg"].to_string();

                let l = event.l.new(o!{
                    "command" => command.clone(),
                    "command_arg" => arg.clone(),
                });
                info!(l, "Handling command");

                let handler = match self.on_command_handlers.get(&command) {
                    Some(handler) => handler,
                    None => {
                        error!(l, "Command does not exist");
                        event.message.reply(&format!("Command {:?} does not exist", command));
                        return
                    }
                };

                handler.recipient.do_send(OnCommand {
                    message: event.message,
                    command,
                    arg,
                    l,
                });

            },
            None => {
                for handler in &self.on_message_handlers {
                    handler.recipient.do_send(event.clone());
                }
            },
        }
    }
}

#[derive(Clone)]
pub struct OnMessageHandler {
    plugin_id: PluginId,
    priority: Priority,
    recipient: Recipient<OnMessage>,
}

#[derive(Clone)]
pub struct OnCommandHandler {
    plugin_id: PluginId,
    recipient: Recipient<OnCommand>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Priority {
    level: i64,
}

impl Priority {
    pub const NORMAL: Self = Priority { level: 1000 };
}

use event::*;

pub mod event {
    use actix::prelude::*;
    use super::{Message, PluginId, PluginContext};
    use std::sync::Arc;
    use slog::Logger;

    #[derive(Message)]
    pub struct RegisterPlugin<N, F, A>
    where
        N: Into<String>,
        F: FnOnce(PluginContext<A>) -> A,
        A: Actor<Context = Context<A>>,
    {
        pub name: N,
        pub f: F,
        _a: std::marker::PhantomData<A>,
    }

    #[derive(Message, Clone)]
    pub struct OnMessage {
        pub message: Arc<Message>,
        pub l: Logger,
    }

    #[derive(Message, Clone)]
    pub struct OnCommand {
        pub message: Arc<Message>,
        pub command: String,
        pub arg: String,
        pub l: Logger,
    }

    #[derive(Message)]
    pub enum UnloadPlugin {
        ById(PluginId),
    }

    #[derive(Message)]
    pub struct RegisterOnMessageHandler {
        pub handler: super::OnMessageHandler,
    }

    #[derive(Message)]
    pub struct RegisterOnCommandHandler {
        pub handler: super::OnCommandHandler,
        pub command: String,
    }
}

