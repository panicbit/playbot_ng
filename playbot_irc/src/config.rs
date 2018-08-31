use irc;
use std::path::Path;
use std::fs;
use failure::Error;
use toml;

#[derive(Deserialize)]
pub struct Config {
    #[serde(rename = "instance")]
    pub instances: Vec<irc::client::data::Config>,
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Error> {
        let config = fs::read_to_string(path)?;
        let config = toml::de::from_str::<Self>(&config)?;
        Ok(config)
    }
}
