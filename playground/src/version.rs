use crate::Channel;
use reqwest::{Client, Error};

pub async fn version(client: &Client, channel: Channel) -> Result<Version, Error> {
    let resp = client
        .get(&format!("https://play.rust-lang.org/meta/version/{}", channel.as_str()))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(resp)
}


#[derive(Deserialize)]
pub struct Version {
    pub date: String,
    pub hash: String,
    pub version: String,
}
