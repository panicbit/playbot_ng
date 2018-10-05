use crate::{Channel, CrateType, Mode};
use std::borrow::Cow;
use reqwest::{Client, Error};
use reqwest::r#async as async_reqwest;
use std::future::Future;
use futures::compat::Future01CompatExt;

pub fn execute(client: &Client, req: &Request) -> Result<Response, Error> {
    let resp = client
        .post("https://play.rust-lang.org/execute")
        .json(req)
        .send()?
        .error_for_status()?
        .json()?;
    
    Ok(resp)
}

pub fn async_execute<'a>(req: &'a Request) -> impl Future<Output = Result<Response, Error>> + 'a {
    async move {
        let url = "https://play.rust-lang.org/execute";
        let client = async_reqwest::Client::new();
        let resp = await!(
            client.post(url)
            .json(req)
            .send()
            .compat()
        )?;
        let mut resp = resp.error_for_status()?;
        let resp = await!(resp.json().compat())?;

        Ok(resp)
    }
}

#[derive(Serialize,Debug)]
#[serde(rename_all = "camelCase")]
pub struct Request<'a> {
    channel: Channel,
    mode: Mode,
    edition: Option<String>,
    crate_type: CrateType,
    tests: bool,
    backtrace: bool,
    code: Cow<'a, str>,
}

impl<'a> Request<'a> {
    pub fn new<S: Into<Cow<'a, str>>>(code: S) -> Self {
        Self {
            code: code.into(),
            channel: Channel::Stable,
            crate_type: CrateType::Bin,
            mode: Mode::Debug,
            edition: None,
            backtrace: false,
            tests: false,
        }
    }

    pub fn new_with<S: Into<Cow<'a, str>>>(code: S, channel: Channel, mode: Mode, edition: Option<String>) -> Self {
        Self {
            code: code.into(),
            channel,
            crate_type: CrateType::Bin,
            mode,
            edition,
            backtrace: false,
            tests: false,
        }
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn set_code(&mut self, code: impl Into<Cow<'a, str>>) {
        self.code = code.into();
    }

    pub fn channel(&self) -> Channel {
        self.channel
    }

    pub fn backtrace(&self) -> bool {
        self.backtrace
    }

    pub fn set_channel(&mut self, channel: Channel) {
        self.channel = channel;
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    pub fn set_backtrace(&mut self, state: bool) {
        self.backtrace = state;
    }

    pub fn edition(self) -> Option<String> {
        self.edition
    }

    pub fn set_edition(&mut self, edition: Option<String>) {
        self.edition = edition;
    }
}

#[derive(Deserialize,Debug)]
pub struct Response {
    pub stderr: String,
    pub stdout: String,
    pub success: bool,
}
