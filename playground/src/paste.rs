use crate::{Channel, Mode};
use reqwest::{Client, Error};

pub async fn paste<S: AsRef<str>>(client: &Client, text: S, channel: Channel, mode: Mode) -> Result<String, Error> {
    let gist_id = client
        .post("https://play.rust-lang.org/meta/gist/")
        .json(&Request::new(text.as_ref()))
        .send()
        .await?
        .error_for_status()?
        .json::<Response>()
        .await?
        .id;

    let url = format!("https://play.rust-lang.org/?gist={gist}&version={channel}&mode={mode}",
        gist = gist_id,
        channel = channel.as_str(),
        mode = mode.as_str()
    );

    Ok(url)
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
