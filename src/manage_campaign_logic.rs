use std::{fs::{File, OpenOptions}, io::{Write, self, ErrorKind}, path::PathBuf};
use serde::{Deserialize, Serialize};
use toml::to_string;
use std::env;
use std::io::Error;

#[derive(Serialize)]
struct Config {
    campaign: Campaign
}

#[derive(Serialize)]
pub struct Campaign {
    pub name: String,
    pub path: String,
    pub sync_option: String
}

pub fn read_campaign_from_config() -> Option<Vec<Campaign>> {
    todo!();
}

pub fn write_campaign_to_config(campaign: Campaign) -> Result<(), io::Error>{
    let config_item = Config{campaign};
    let mut config_file = get_campaign_config("append")?;
    let toml_string = to_string(&config_item).unwrap();
    config_file.write_all(toml_string.as_bytes())?;
    Ok(())
}


pub fn remove_campaign_from_config(campaign: Campaign) -> bool {
    todo!();
}

fn get_campaign_config(operation: &str) -> Result<File, io::Error>{
    let mut path = env::current_dir()?;
    path.push(".config.toml");
    match operation {
        "read" => {
            let file = OpenOptions::new()
                .read(true)
                .open(&path)?;
            return Ok(file)
        },
        "write" => {
            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(&path)?;
            return Ok(file)
        },
        "append" => {
            let file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(&path)?;
            return Ok(file)
        },
        _ => Err(Error::from(ErrorKind::InvalidInput))
    }
}