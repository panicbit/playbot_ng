use ::playground::{self, ExecuteRequest, Channel, Mode, CrateType};
use regex::Regex;
use reqwest::Client;
use actix::prelude::*;
use super::*;
use crate::Message;
use std::borrow::Cow;

lazy_static! {
    static ref CRATE_ATTRS: Regex = Regex::new(r"^(\s*#!\[.*?\])*").unwrap();
}

pub(crate) struct Playground {}

impl Playground {
    pub fn new(ctx: PluginContext<Self>) -> Self {
        ctx.on_message(Priority::NORMAL, ctx.recipient());
        ctx.on_command("eval", ctx.recipient());
        Self {}
    }
}

impl Actor for Playground {
    type Context = Context<Self>;
}

impl Handler<OnMessage> for Playground {
    type Result = ();

    fn handle(&mut self, event: OnMessage, ctx: &mut Context<Self>) {
        if !event.message.is_directly_addressed() {
            return;
        }

        execute_code(&*event.message, &event.message.body());
    }
}

impl Handler<OnCommand> for Playground {
    type Result = ();

    fn handle(&mut self, event: OnCommand, ctx: &mut Context<Self>) {
        if event.command != "eval" {
            return;
        }

        execute_code(&*event.message, &event.arg);
    }
}


#[derive(PartialEq)]
enum Template {
    Expr,
    Bare,
    ExprAllocStats,
}

fn execute_code(message: &Message, mut body: &str) {
    let mut request = ExecuteRequest::new("");
    let mut template = Template::Expr;

    // Parse flags
    loop {
        body = body.trim_start();
        let flag = body.split_whitespace().next().unwrap_or("");

        match flag {
            "--stable" => request.set_channel(Channel::Stable),
            "--beta" => request.set_channel(Channel::Beta),
            "--nightly" => request.set_channel(Channel::Nightly),
            "--version" | "VERSION" => {
                print_version(request.channel(), &*message);
                return;
            },
            "--bare" | "--mini" => template = Template::Bare,
            "--allocs" | "--alloc" | "--stats" | "--alloc-stats" => template = Template::ExprAllocStats,
            "--debug" => request.set_mode(Mode::Debug),
            "--release" => request.set_mode(Mode::Release),
            "--2015" => request.set_edition(Some("2015".to_owned())),
            "--2018" => request.set_edition(Some("2018".to_owned())),
            "help" | "h" | "-h" | "-help" | "--help" | "--h" => {
                super::help::display_help(message);
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

    let mut body = Cow::Borrowed(body.trim_start());

    if gist::is_gist_or_raw_gist_url(&body) {
        template = Template::Bare;
        match gist::fetch_gist(&body) {
            Ok(file) => body = Cow::Owned(file),
            Err(e) => {
                eprintln!("[ERR/gist/{}]: {}", body, e);
                message.reply("Failed to fetch gist");
                return;
            }
        }
    }

    if template == Template::Bare {
        if let Ok(syn::File { attrs, items, .. }) = syn::parse_str::<syn::File>(&body) {
            let main_exists = items.iter().any(|item| match item {
                syn::Item::Fn(fun) => fun.ident == "main",
                _ => false,
            });

            if !main_exists {
                request.set_crate_type(CrateType::Lib);
            }

            for attr in attrs {
                match attr.parse_meta().unwrap() {
                    syn::Meta::NameValue(syn::MetaNameValue { ident, lit: syn::Lit::Str(lit_str), .. }) => {
                        if ident != "crate_type" { continue; }

                        match lit_str.value().as_str() {
                            "bin" => request.set_crate_type(CrateType::Bin),
                            "lib" => request.set_crate_type(CrateType::Lib),
                            _ => (),
                        }
                    },
                    _ => (),
                }
            }
        };
    }

    let code = match template {
        Template::Bare => body.to_string(),
        Template::Expr => {
            let crate_attrs = CRATE_ATTRS.find(&body)
                .map(|attr| attr.as_str())
                .unwrap_or("");

            let body_code = &body[crate_attrs.len()..];

            format!(include_str!("../../template.rs"),
                crate_attrs = crate_attrs,
                code = body_code,
            )
        },
        Template::ExprAllocStats => {
            let crate_attrs = CRATE_ATTRS.find(&body)
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
    execute(&*message, &request);
}

fn print_version<'a>(channel: Channel, message: &Message) {
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

    message.reply(&version);
}

pub fn execute(message: &Message, request: &ExecuteRequest) {
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
        message.reply(line);
    }

    if lines_count == 0 && resp.success {
        message.reply("~~~ Code compiled successfully without output.");
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

        message.reply(&format!("~~~ Full output: {}", url));
    }
}

mod gist {
    use regex::Regex;
    use std::error::Error;
    use std::collections::HashMap;

    lazy_static! {
        static ref GIST_URL_RE: Regex = Regex::new(
            "^(https?://)?gist.github.com/([^/ ]+/)?(?P<id>[0-9a-f]+)/?$"
        ).unwrap();

        static ref RAW_GIST_URL_RE: Regex = Regex::new(
            "^(https?://)?gist.githubusercontent.com/[^/ ]+/[0-9a-f]+/raw(/.*)?"
        ).unwrap();
    }

    pub fn is_gist_or_raw_gist_url(url: &str) -> bool {
        RAW_GIST_URL_RE.is_match(url) || GIST_URL_RE.is_match(url)
    }

    pub fn fetch_gist(url: &str) -> Result<String, Box<Error>> {
        let url = url.trim();

        if RAW_GIST_URL_RE.is_match(url) {
            let mut file = reqwest::get(url)?;
            let file = file.text()?;
            return Ok(file);
        }

        let captures = GIST_URL_RE.captures(url)
            .ok_or("Not a gist url")?;
        let id = &captures["id"];
        let url = format!("https://api.github.com/gists/{}", id);
        let gist = reqwest::get(&url)?.json::<Gist>()?;
        let file = gist.files.into_iter()
            .map(|(_, file)| file)
            .find(|file| file.filename.ends_with(".rs"))
            .ok_or("No .rs file found in the gist")?;

        Ok(file.content)
    }

    #[derive(Deserialize)]
    struct Gist {
        files: HashMap<String, File>,
    }

    #[derive(Deserialize)]
    struct File {
        filename: String,
        content: String,
    }
}
