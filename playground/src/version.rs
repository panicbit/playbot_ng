use crate::Channel;
use reqwest::{Client, Error};
use reqwest::r#async as async_reqwest;
use futures::prelude::*;

pub fn version(client: &Client, channel: Channel) -> Result<Version, Error> {
    let resp = client
        .get(&format!("https://play.rust-lang.org/meta/version/{}", channel.as_str()))
        .send()?
        .error_for_status()?
        .json()?;

    Ok(resp)
}

pub fn async_version(channel: Channel) -> impl Future<Item = Version, Error = Error> {
    let client = async_reqwest::Client::new();
    let url = format!("https://play.rust-lang.org/meta/version/{}", channel.as_str());

    client
    .get(&url)
    .send()
    .and_then(|resp| resp.error_for_status())
    .and_then(|mut resp| resp.json())
}

#[derive(Deserialize)]
pub struct Version {
    pub date: String,
    pub hash: String,
    pub version: String,
}
