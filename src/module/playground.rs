use crate::module::prelude::*;
use playground::{self, ExecuteRequest, Channel, Mode};
use reqwest::{self, Client};
use regex::Regex;
use futures::future::LocalFutureObj;
use futures::prelude::*;
use tokio_core::reactor::Handle;

lazy_static! {
    static ref CRATE_ATTRS: Regex = Regex::new(r"^(\s*#!\[.*?\])*").unwrap();
}

pub enum Playground {}

impl Module for Playground {
    fn init(commands: &mut CommandRegistry) {
        commands.add_fallback_handler(playground_handler);
    }
}

fn playground_handler<'a>(handle: Handle, ctx: &'a Context) -> LocalFutureObj<'a, Flow> {
    LocalFutureObj::new((async move || {
        let http = reqwest::Client::new();

        if !ctx.is_directly_addressed() {
            return Flow::Continue;
        }

        let mut body = ctx.body();
        let mut channel = Channel::Stable;
        let mut show_version = false;
        let mut bare = false;
        let mut mode = Mode::Debug;

        // Parse flags
        loop {
            body = body.trim_left();
            let flag = body.split_whitespace().next().unwrap_or("");

            match flag {
                "--stable" => channel = Channel::Stable,
                "--beta" => channel = Channel::Beta,
                "--nightly" => channel = Channel::Nightly,
                "--version" | "VERSION" => show_version = true,
                "--bare" | "--mini" => bare = true,
                "--debug" => mode = Mode::Debug,
                "--release" => mode = Mode::Release,
                "help" | "h" | "-h" | "-help" | "--help" | "--h" => {
                    super::help::display_help(ctx);
                    return Flow::Break;
                }
                _ => break,
            }

            body = &body[flag.len()..];
        }

        if show_version {
            await!(print_version(handle, channel, &ctx));
            return Flow::Break;
        }

        let code = if bare { body.to_string() } else {
            let crate_attrs = CRATE_ATTRS.find(body)
                .map(|attr| attr.as_str())
                .unwrap_or("");

            body = &body[crate_attrs.len()..];

            format!(include_str!("../../template.rs"),
                crate_attrs = crate_attrs,
                code = body,
            )
        };

        let mut request = ExecuteRequest::new(code.as_str());
        request.set_channel(channel);
        request.set_mode(mode);

        execute(&ctx, &http, &request);

        Flow::Break
    })().boxed())
}

fn print_version<'a>(handle: Handle, channel: Channel, ctx: &'a Context) -> impl Future<Output = ()> + 'a {
    (async move || {
        let resp = match await!(playground::async_version(handle, channel)) {
            Err(e) => return eprintln!("Failed to get version: {:?}", e),
            Ok(resp) => resp,
        };

        let version = format!("{version} ({hash:.9} {date})",
            version = resp.version,
            hash = resp.hash,
            date = resp.date,
        );

        ctx.reply(version);
    })()
}

pub fn execute(ctx: &Context, http: &Client, request: &ExecuteRequest) {
    let resp = match playground::execute(http, &request) {
        Ok(resp) => resp,
        Err(e) => return {
            eprintln!("Failed to execute code: {:?}", e);
        },
    };

    let output = if resp.success { &resp.stdout } else { &resp.stderr };
    let take_count = if resp.success { 2 } else { 1 };
    let lines = output
        .lines()
        .filter(|line| {
            if resp.success {
                return true;
            }

               !line.trim().starts_with("Compiling")
            && !line.trim().starts_with("Finished")
            && !line.trim().starts_with("Running")
        })
        .take(take_count);

    for line in lines {
        ctx.reply(line);
    }

    if output.lines().count() > take_count {
        let code = format!(include_str!("../../paste_template.rs"),
            code = request.code(),
            stdout = resp.stdout,
            stderr = resp.stderr,
        );

        let url = match playground::paste(http, code, request.channel(), request.mode()) {
            Ok(url) => url,
            Err(e) => return {
                eprintln!("Failed to paste code: {:?}", e);
            },
        };

        ctx.reply(format!("~~~ Full output: {}", url));
    }
}
