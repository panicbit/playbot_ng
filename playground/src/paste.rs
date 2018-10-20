use crate::{Channel, Mode};
use reqwest::{Client, Error};
use reqwest::r#async as async_reqwest;
use futures::prelude::*;

pub fn paste<S: AsRef<str>>(client: &Client, text: S, channel: Channel, mode: Mode) -> Result<String, Error> {
    let gist_id = client
        .post("https://play.rust-lang.org/meta/gist/")
        .json(&Request::new(text.as_ref()))
        .send()?
        .error_for_status()?
        .json::<Response>()?
        .id;

    let url = format!("https://play.rust-lang.org/?gist={gist}&version={channel}&mode={mode}",
        gist = gist_id,
        channel = channel.as_str(),
        mode = mode.as_str()
    );

    Ok(url)
}

pub fn async_paste(text: impl Into<String>, channel: Channel, mode: Mode) -> impl Future<Item = String, Error = Error> {
    let text = text.into();
    let client = async_reqwest::Client::new();
    let url = "https://play.rust-lang.org/meta/gist/";

    client
    .post(url)
    .json(&Request::new(text.as_ref()))
    .send()
    .and_then(|resp| resp.error_for_status())
    .and_then(|mut resp| resp.json::<Response>())
    .map(move |gist| format!("https://play.rust-lang.org/?gist={gist_id}&version={channel}&mode={mode}",
        gist_id = gist.id,
        channel = channel.as_str(),
        mode = mode.as_str()
    ))
}

#[derive(Serialize)]
struct Request<'a> {
    code: &'a str,
}

impl<'a> Request<'a> {
    fn new(code: &'a str) -> Self {
        Request { code }
    }
}

#[derive(Deserialize)]
struct Response {
    id: String,
}
