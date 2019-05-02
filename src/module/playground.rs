use crate::module::prelude::*;
use crate::Command;
use playground::{self, ExecuteRequest, Channel, Mode, CrateType};
use regex::Regex;
use reqwest::Client;

lazy_static! {
    static ref CRATE_ATTRS: Regex = Regex::new(r"^(\s*#!\[.*?\])*").unwrap();
}

pub(crate) enum Playground {}

impl Module for Playground {
    fn init(commands: &mut CommandRegistry) {
        commands.add_fallback_handler(playground_fallback_handler);
        commands.set_named_handler("eval", playground_named_handler);
    }
}

#[derive(PartialEq)]
enum Template {
    Expr,
    Bare,
    ExprAllocStats,
}

fn playground_fallback_handler(ctx: &Context) {
    if !ctx.is_directly_addressed() {
        return;
    }

    execute_code(ctx, ctx.body());
}

fn playground_named_handler(ctx: &Context, cmd: &Command) {
    execute_code(&ctx, cmd.raw_args());
}

fn execute_code(ctx: &Context, mut body: &str) {
    let mut request = ExecuteRequest::new("");
    let mut template = Template::Expr;

    // Parse flags
    loop {
        body = body.trim_left();
        let flag = body.split_whitespace().next().unwrap_or("");

        match flag {
            "--stable" => request.set_channel(Channel::Stable),
            "--beta" => request.set_channel(Channel::Beta),
            "--nightly" => request.set_channel(Channel::Nightly),
            "--version" | "VERSION" => {
                print_version(request.channel(), &ctx);
                return;
            },
            "--bare" | "--mini" => template = Template::Bare,
            "--allocs" | "--alloc" | "--stats" | "--alloc-stats" => template = Template::ExprAllocStats,
            "--debug" => request.set_mode(Mode::Debug),
            "--release" => request.set_mode(Mode::Release),
            "--2015" => request.set_edition(Some("2015".to_owned())),
            "--2018" => request.set_edition(Some("2018".to_owned())),
            "help" | "h" | "-h" | "-help" | "--help" | "--h" => {
                super::help::display_help(ctx);
                return;
            },
            "--" => {
                body = &body[flag.len()..];
                break;
            },
            _ => break,
        }

        body = &body[flag.len()..];
    }

    body = body.trim_left();

    if template == Template::Bare {
        if let Ok(syn::File { items, .. }) = syn::parse_str::<syn::File>(body) {
            let main_exists = items.iter().any(|item| match item {
                syn::Item::Fn(fun) => fun.ident == "main",
                _ => false,
            });

            if !main_exists {
                request.set_crate_type(CrateType::Lib);
            }
        };
    }

    let code = match template {
        Template::Bare => body.to_string(),
        Template::Expr => {
            let crate_attrs = CRATE_ATTRS.find(body)
                .map(|attr| attr.as_str())
                .unwrap_or("");

            let body_code = &body[crate_attrs.len()..];

            format!(include_str!("../../template.rs"),
                crate_attrs = crate_attrs,
                code = body_code,
            )
        },
        Template::ExprAllocStats => {
            let crate_attrs = CRATE_ATTRS.find(body)
                .map(|attr| attr.as_str())
                .unwrap_or("");

            let body_code = &body[crate_attrs.len()..];

            format!(include_str!("../../alloc_stats_template.rs"),
                crate_attrs = crate_attrs,
                code = body_code,
            )
        },
    };

    request.set_code(code);
    execute(&ctx, &request);
}

fn print_version<'a>(channel: Channel, ctx: &'a Context) {
    let http = Client::new();
    let resp = match playground::version(&http, channel) {
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

pub fn execute<'a>(ctx: &'a Context, request: &'a ExecuteRequest) {
    let http = Client::new();
    let resp = match playground::execute(&http, &request) {
        Ok(resp) => resp,
        Err(e) => return {
            eprintln!("Failed to execute code: {:?}", e);
        },
    };

    let output = if resp.success { &resp.stdout } else { &resp.stderr };
    let take_count = if resp.success { 3 } else { 1 };
    let lines = output
        .lines()
        .filter(|line| {
            if resp.success {
                return true;
            }

            !line.trim().starts_with("Compiling")
            && !line.trim().starts_with("Finished")
            && !line.trim().starts_with("Running")
        });
    let lines_count = lines.clone().count();

    for line in lines.take(take_count) {
        ctx.reply(line);
    }

    if lines_count == 0 && resp.success {
        ctx.reply("~~~ Code compiled successfully without output.");
    }

    if lines_count > take_count {
        let code = format!(include_str!("../../paste_template.rs"),
            code = request.code(),
            stdout = resp.stdout,
            stderr = resp.stderr,
        );

        let url = match playground::paste(&http, code, request.channel(), request.mode()) {
            Ok(url) => url,
            Err(e) => return {
                eprintln!("Failed to paste code: {:?}", e);
            },
        };

        ctx.reply(format!("~~~ Full output: {}", url));
    }
}
