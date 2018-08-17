use crate::{Channel, Mode};
use reqwest::{Client, Error};
use reqwest::unstable::r#async as async_reqwest;
use futures::prelude::*;
use futures::compat::Future01CompatExt;
use tokio_core::reactor::Handle;

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

pub fn async_paste<S: AsRef<str>>(handle: Handle, text: S, channel: Channel, mode: Mode) -> impl Future<Output = Result<String, Error>> {
    (async move || {
        let client = async_reqwest::Client::new(&handle);
        let url = "https://play.rust-lang.org/meta/gist/";
        let resp = await!(
            client
                .post(url)
                .json(&Request::new(text.as_ref()))
                .send()
                .compat()
        )?;
        let mut resp = resp.error_for_status()?;
        let gist_id = await!(resp.json::<Response>().compat())?.id;

        let url = format!("https://play.rust-lang.org/?gist={gist}&version={channel}&mode={mode}",
            gist = gist_id,
            channel = channel.as_str(),
            mode = mode.as_str()
        );

        Ok(url)
    })()
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
