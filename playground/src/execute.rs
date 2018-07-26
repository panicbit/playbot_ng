use {Channel, CrateType, Mode};
use std::borrow::Cow;
use reqwest::{Client, Error};

pub fn execute(client: &Client, req: &Request) -> Result<Response, Error> {
    let resp = client
        .post("https://play.rust-lang.org/execute")
        .json(req)
        .send()?
        .error_for_status()?
        .json()?;
    
    Ok(resp)
}

#[derive(Serialize,Debug)]
#[serde(rename_all = "camelCase")]
pub struct Request<'a> {
    channel: Channel,
    mode: Mode,
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
            backtrace: false,
            tests: false,
        }
    }

    pub fn code(&self) -> &str {
        &self.code
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
}

#[derive(Deserialize,Debug)]
pub struct Response {
    pub stderr: String,
    pub stdout: String,
    pub success: bool,
}
