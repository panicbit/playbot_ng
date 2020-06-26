use ::playground::{self, ExecuteRequest, Channel, Mode, CrateType};
use regex::Regex;
use reqwest::Client;
use actix::prelude::*;
use futures::prelude::*;
use super::*;
use crate::Message;
use std::borrow::Cow;
use slog::Logger;

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
    type Result = ResponseFuture<()>;

    fn handle(&mut self, event: OnMessage, _ctx: &mut Context<Self>) -> Self::Result {
        async move {
            if !event.message.is_directly_addressed() {
                return;
            }

            execute_code(&*event.message, &event.message.body(), &event.l).await;
        }
        .boxed()
    }
}

impl Handler<OnCommand> for Playground {
    type Result = ResponseFuture<()>;

    fn handle(&mut self, event: OnCommand, _ctx: &mut Context<Self>) -> Self::Result {
        async move {
            if event.command != "eval" {
                return;
            }

            execute_code(&*event.message, &event.arg, &event.l).await;
        }
        .boxed()
    }
}


#[derive(PartialEq)]
enum Template {
    Expr,
    Bare,
    ExprAllocStats,
}

async fn execute_code(message: &dyn Message, mut body: &str, l: &Logger) {
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
                print_version(request.channel(), &*message).await;
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
        match gist::fetch_gist(&body).await {
            Ok(file) => body = Cow::Owned(file),
            Err(e) => {
                error!(l, "[ERR/gist/{}]: {}", body, e);
                message.reply("Failed to fetch gist");
                return;
            }
        }
    }

    if template == Template::Bare {
        if let Ok(syn::File { attrs, items, .. }) = syn::parse_str::<syn::File>(&body) {
            let main_exists = items.iter().any(|item| match item {
                syn::Item::Fn(fun) => fun.sig.ident == "main",
                _ => false,
            });

            if !main_exists {
                request.set_crate_type(CrateType::Lib);
            }

            for attr in attrs {
                match attr.parse_meta().unwrap() {
                    syn::Meta::NameValue(syn::MetaNameValue { path, lit: syn::Lit::Str(lit_str), .. }) => {
                        let ident = match path.get_ident() {
                            Some(ident) => ident,
                            None => continue,
                        };

                        if ident != "crate_type" {
                            continue;
                        }

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
    execute(&*message, &request).await;
}

async fn print_version<'a>(channel: Channel, message: &dyn Message) {
    let http = Client::new();
    let resp = match playground::version(&http, channel).await {
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

pub async fn execute(message: &dyn Message, request: &ExecuteRequest<'_>) {
    let http = Client::new();
    let resp = match playground::execute(&http, &request).await {
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

        let url = match playground::paste(&http, code, request.channel(), request.mode()).await {
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

    pub async fn fetch_gist(url: &str) -> Result<String, Box<dyn Error>> {
        let url = url.trim();

        if RAW_GIST_URL_RE.is_match(url) {
            let file = reqwest::get(url).await?.error_for_status()?;
            let file = file.text().await?;
            return Ok(file);
        }

        let captures = GIST_URL_RE.captures(url)
            .ok_or("Not a gist url")?;
        let id = &captures["id"];
        let url = format!("https://api.github.com/gists/{}", id);
        let gist = reqwest::get(&url).await?.error_for_status()?.json::<Gist>().await?;
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
