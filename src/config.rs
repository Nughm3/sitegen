use color_eyre::eyre::Result;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{default::Default, fs, io::Read, path::PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub name: String,
    pub address: String,
    pub port: u16,
    pub threads: u8,
    pub index_page: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            name: String::from("sitegen-project"),
            address: String::from("127.0.0.1"),
            port: 8000,
            threads: 4,
            index_page: PathBuf::new(),
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Config::default()
    }
    pub fn with_name(name: String) -> Self {
        Config {
            name,
            ..Default::default()
        }
    }
}

pub fn configure(name: String) -> Result<()> {
    let config = Config::with_name(name);
    let config = serde_json::to_string(&config)?;
    fs::write("server.json", &config)?;
    Ok(())
}

pub fn load(f: &mut fs::File) -> Result<Config> {
    let mut config = String::new();
    f.read_to_string(&mut config)?;
    let config = serde_json::from_str(&config)?;
    Ok(config)
}
