use crate::module::prelude::*;
use playground::{self, ExecuteRequest, Channel, Mode};
use regex::Regex;
use futures::future::LocalFutureObj;
use futures::prelude::*;

lazy_static! {
    static ref CRATE_ATTRS: Regex = Regex::new(r"^(\s*#!\[.*?\])*").unwrap();
}

pub(crate) enum Playground {}

impl Module for Playground {
    fn init(commands: &mut CommandRegistry) {
        commands.add_fallback_handler(playground_handler);
    }
}

fn playground_handler<'a>(ctx: &'a Context) -> LocalFutureObj<'a, Flow> {
    LocalFutureObj::new(async move {
        if !ctx.is_directly_addressed() {
            return Flow::Continue;
        }

        let mut body = ctx.body();
        let mut channel = Channel::Stable;
        let mut show_version = false;
        let mut bare = false;
        let mut mode = Mode::Debug;
        let mut edition = None;

        // Parse flags
        loop {
            body = body.trim_left();
            let flag = body.split_whitespace().next();

            match flag {
                Some("--stable") => channel = Channel::Stable,
                Some("--beta") => channel = Channel::Beta,
                Some("--nightly") => channel = Channel::Nightly,
                Some("--version") | Some("VERSION") => show_version = true,
                Some("--bare") | Some("--mini") => bare = true,
                Some("--debug") => mode = Mode::Debug,
                Some("--release") => mode = Mode::Release,
                Some("--2015") => edition = Some("2015".to_owned()),
                Some("--2018") => edition = Some("2018".to_owned()),
                Some("help") | Some("h") | Some("-h") | Some("-help") | Some("--help") | Some("--h") => {
                    super::help::display_help(ctx);
                    return Flow::Break;
                }
                _ => break,
            }

            body = match flag {
                Some(f) => &body[f.len()..],
                None => body,
            };
        }

        if show_version {
            await!(print_version(channel, &ctx));
            return Flow::Break;
        }

        let code = if bare { body.to_string() } else {
            let crate_attrs = CRATE_ATTRS.find(body)
                .map(|attr| attr.as_str())
                .unwrap_or("");

            let body_code = &body[crate_attrs.len()..];

            format!(include_str!("../../template.rs"),
                crate_attrs = crate_attrs,
                code = body_code,
            )
        };

        let request = ExecuteRequest::new_with(&code, channel, mode, edition);

        await!(execute(&ctx, &request));

        Flow::Break
    }.boxed())
}

fn print_version<'a>(channel: Channel, ctx: &'a Context) -> impl Future<Output = ()> + 'a {
    async move {
        let resp = match await!(playground::async_version(channel)) {
            Err(e) => return eprintln!("Failed to get version: {:?}", e),
            Ok(resp) => resp,
        };

        let version = format!("{version} ({hash:.9} {date})",
            version = resp.version,
            hash = resp.hash,
            date = resp.date,
        );

        ctx.reply(version);
    }
}

pub fn execute<'a>(ctx: &'a Context, request: &'a ExecuteRequest) -> impl Future<Output = ()> + 'a {
    async move {
        let resp = match await!(playground::async_execute(&request)) {
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

            let url = match await!(playground::async_paste(code, request.channel(), request.mode())) {
                Ok(url) => url,
                Err(e) => return {
                    eprintln!("Failed to paste code: {:?}", e);
                },
            };

            ctx.reply(format!("~~~ Full output: {}", url));
        }
    }
}
