use crate::Channel;
use reqwest::{Client, Error};

use reqwest::unstable::r#async as async_reqwest;
use futures::prelude::*;
use futures::compat::Future01CompatExt;
use tokio_core::reactor::Handle;

pub fn version(client: &Client, channel: Channel) -> Result<Version, Error> {
    let resp = client
        .get(&format!("https://play.rust-lang.org/meta/version/{}", channel.as_str()))
        .send()?
        .error_for_status()?
        .json()?;

    Ok(resp)
}

pub fn async_version(handle: Handle, channel: Channel) -> impl Future<Output = Result<Version, Error>> {
    (async || {
        let client = async_reqwest::Client::new(&handle);
        let url = format!("https://play.rust-lang.org/meta/version/{}", channel.as_str());

        let resp = await!(client.get(&url).send().compat())?;
        let mut resp = resp.error_for_status()?;
        let resp = await!(resp.json().compat())?;

        Ok(resp)
    })()
}

#[derive(Deserialize)]
pub struct Version {
    pub date: String,
    pub hash: String,
    pub version: String,
}
