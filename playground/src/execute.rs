use crate::{Channel, CrateType, Mode};
use std::borrow::Cow;
use reqwest::{Client, Error};

pub async fn execute(client: &Client, req: &Request<'_>) -> Result<Response, Error> {
    let resp = client
        .post("https://play.rust-lang.org/execute")
        .json(req)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(resp)
}

#[derive(Serialize,Debug)]
#[serde(rename_all = "camelCase")]
pub struct Request<'a> {
    channel: Channel,
    mode: Mode,
    #[serde(skip_serializing_if = "Option::is_none")]
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
            edition: Some("2018".into()),
            backtrace: false,
            tests: false,
        }
    }

    pub fn new_with<S: Into<Cow<'a, str>>>(code: S, channel: Channel, mode: Mode, edition: Option<String>, crate_type: CrateType) -> Self {
        Self {
            code: code.into(),
            channel,
            crate_type,
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

    pub fn crate_type(&self) -> CrateType {
        self.crate_type
    }

    pub fn set_crate_type(&mut self, crate_type: CrateType) {
        self.crate_type = crate_type;
    }
}

#[derive(Deserialize,Debug)]
pub struct Response {
    pub stderr: String,
    pub stdout: String,
    pub success: bool,
}
