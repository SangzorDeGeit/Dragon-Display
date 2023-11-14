use std::{fs::{File, OpenOptions}, io::{Write, self, ErrorKind, Read}};
use serde::{Deserialize, Serialize};
use toml::to_string;
use std::env;
use std::io::Error;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
struct Config {
    campaigns : HashMap<String, CampaignData>
}

#[derive(Serialize, Deserialize)]
pub struct CampaignData {
    pub path : String,
    pub sync_option : String
}

pub fn read_campaign_from_config() -> Option<HashMap<String, CampaignData>> {
    let mut file = match get_campaign_config("read") {
        Ok(file) => file,
        Err(_) => {
            return None
        }
    };

    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(_) => {
            return None
        }
    };
    println!("contents: \n{}", contents);
    let config: Config = match toml::from_str(&contents) {
        Ok(d) => d,
        Err(_) => {
            return None
        }
    };
    return Some(config.campaigns);
}


pub fn write_campaign_to_config(campaign: HashMap<String, CampaignData>) -> Result<(), io::Error>{
    let config_item = Config{campaigns: campaign};
    let mut config_file = get_campaign_config("append")?;
    let toml_string = to_string(&config_item).unwrap();
    config_file.write_all(toml_string.as_bytes())?;
    Ok(())
}


pub fn remove_campaign_from_config(campaign: HashMap<String, CampaignData>) -> Result<(), io::Error> {
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