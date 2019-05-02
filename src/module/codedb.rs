use syn;
use reqwest::Client;
use super::{Flow, Context};

pub struct CodeDB<'a> {
    db: &'a mut ::codedb::CodeDB,
    http: &'a Client,
}

impl<'a> CodeDB<'a> {
    pub fn new(db: &'a mut ::codedb::CodeDB, http: &'a Client) -> Self {
        Self { db, http }
    }
}

impl<'a> Module for CodeDB<'a> {
    fn run(&mut self, ctx: Context) -> Flow {
        let mut body = ctx.body();
        let mut overwrite = false;

        // Ensure the prefix exists
        if !body.starts_with("#!") {
            return Flow::Continue;
        }
        body = &body[2..];

        // Check for overwrite bang
        if body.starts_with("!") {
            overwrite = true;
            body = &body[1..];
        }
        body = body.trim_start();

        // Check if we are defining a fn
        if body.starts_with("fn") {
            self.define_fn(&ctx, body, overwrite);
        } else {
            self.run_fn(&ctx, body);
        }

        Flow::Break
    }
}

impl<'a> CodeDB<'a> {
    fn define_fn(&mut self, ctx: &Context, mut body: &str, overwrite: bool) {
        use syn::Item;
        use syn::ItemKind::Fn;

        body = body.trim();

        let fun_item = match syn::parse_items(body) {
            Ok(ref mut items) if items.len() >= 1 => {
                items.swap_remove(0)
            },
            Ok(_) => {
                ctx.reply("Expected at least one fn item");
                return;
            },
            Err(e) => {
                ctx.reply(e.to_string());
                return;
            }
        };

        let name = match fun_item {
            Item { ident, node: Fn(..), .. } => {
                ident.to_string()
            },
            _ => {
                ctx.reply("Only fns are allowed");
                return;
            },
        };

        let exists = self.db.lookup_fn(&name).is_some();
        if exists && !overwrite {
            ctx.reply(format!("'{}' already exists. Use #!! to overwrite.", name));
            return;
        }

        match self.db.insert_fn(name.as_str(), body, ctx.source()) {
            Ok(_) => ctx.reply(format!("Defined '{}'", name)),
            Err(e) => {
                println!("# Failed to define '{}'.", name);
                println!("| Definition: {}", body);
                println!("| Error: {}", e);
            }
        };
    }

    fn run_fn(&self, ctx: &Context, call: &str) {
        use syn::{Expr, ExprKind};
        use syn::Path;

        let name = match syn::parse_expr(call) {
            Ok(Expr {
                node: ExprKind::Call(
                    box Expr {
                        node: ExprKind::Path(
                            None,
                            Path {
                                global: false,
                                ref segments,
                            },
                        ),
                        ..
                    },
                    _
                ),
                ..
            }) if segments.len() >= 1 => {
                segments[0].ident.to_string()
            },
            _ => {
                ctx.reply("Invalid fn call");
                return;
            }
        };

        if let Some(fun) = self.db.lookup_fn(&name) {
            let code = format!(include!("../../codedb_template.rs"),
                fun = fun,
                call = call,
            );

            ctx.reply(format!("Running: {}", fun));

            let req = ::playground::ExecuteRequest::new(code);
            super::playground::execute(ctx, self.http, &req);
        } else {
            ctx.reply(format!("'{}' does not exist.", name));
        }
    }
}
