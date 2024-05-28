use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Error, ErrorKind, Read, Write},
};
use toml::to_string;

use crate::manage_campaign::MAX_CAMPAIGN_AMOUNT;

enum Operation {
    READ,
    WRITE,
    APPEND,
}

/// Structure representing the name of the campaign and the corresponding data
/// # Example
/// A .config.toml file containing:  
/// ```
/// [campaigns.adventure]  
/// sync_option: "None"  
/// path: "path/to/file"
/// [campaigns.adventure2]
/// sync_option: "google_drive"
/// path: "path/to/file"
/// access_token: "acess_token"
/// refresh_token: "refresh_token"  
/// ```  
/// Will be structured as a hashmap with two key-value pairs. the first key "adventure",
/// with value the campaignData under it until '\[campaigns.adventure2\]'.
/// As second key "adventure2" with as value the campaignData under that.
#[derive(Serialize, Deserialize)]
struct Config {
    campaigns: HashMap<String, CampaignData>,
}

#[derive(Serialize, Deserialize)]
pub struct CampaignData {
    pub sync_option: String,
    pub path: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
}

/// Tries to read the campaign configurations from the config file and puts them in a hashmap.
pub fn read_campaign_from_config() -> Result<HashMap<String, CampaignData>, Error> {
    let mut file = get_campaign_config(Operation::READ)?;

    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {}
        Err(_) => return Err(Error::from(ErrorKind::Unsupported)),
    };

    let config: Config = match toml::from_str(&contents) {
        Ok(d) => d,
        Err(_) => return Err(Error::from(ErrorKind::InvalidData)),
    };

    check_integrity(&config)?;

    Ok(config.campaigns)
}

/// Given a hashmap with the campaign name as key and corresponding campaigndata as value, this function will try to write the campaign to the config file.
pub fn write_campaign_to_config(campaign: HashMap<String, CampaignData>) -> Result<(), io::Error> {
    let config_item = Config {
        campaigns: campaign,
    };
    let mut config_file = get_campaign_config(Operation::APPEND)?;
    let toml_string = to_string(&config_item).unwrap();
    config_file.write_all(toml_string.as_bytes())?;
    Ok(())
}

/// Given an existing campaign name this function will remove this campaign and all the campaigndata from the config file.
pub fn remove_campaign_from_config(campaign_name: &str) -> Result<(), io::Error> {
    let campaign_list = read_campaign_from_config()?;

    let mut new_campaign_list = HashMap::new();

    if campaign_list.len() > 1 {
        for campaign in campaign_list {
            if campaign.0.as_str() != campaign_name {
                new_campaign_list.insert(campaign.0, campaign.1);
            }
        }

        let config_item = Config {
            campaigns: new_campaign_list,
        };
        let mut config_file = get_campaign_config(Operation::WRITE)?;
        let toml_string = to_string(&config_item).unwrap();
        config_file.write_all(toml_string.as_bytes())?;
        Ok(())
    } else if campaign_list.len() == 1 {
        remove_campaign_config()?;
        Ok(())
    } else {
        Err(Error::from(ErrorKind::NotFound))
    }
}

/// Given a file operation this function returns the file with the option for the inputted operation
fn get_campaign_config(operation: Operation) -> Result<File, io::Error> {
    let mut path = env::current_dir()?;
    path.push(".config.toml");
    match operation {
        Operation::READ => {
            let file = OpenOptions::new().read(true).open(&path)?;
            Ok(file)
        }
        Operation::WRITE => {
            let file = OpenOptions::new().write(true).truncate(true).open(&path)?;
            Ok(file)
        }
        Operation::APPEND => {
            let file = OpenOptions::new().append(true).create(true).open(&path)?;
            Ok(file)
        }
    }
}

/// Tries to remove the campaign config file
fn remove_campaign_config() -> Result<(), io::Error> {
    let mut path = env::current_dir()?;
    path.push(".config.toml");
    fs::remove_file(&path)?;
    Ok(())
}

/// Checks for the integrity of the config file. Checks if there are no more campaigns in the file than MAX_CAMPAIGN_AMOUNT
fn check_integrity(config: &Config) -> Result<(), io::Error> {
    if config.campaigns.len() > usize::from(MAX_CAMPAIGN_AMOUNT) {
        return Err(Error::from(ErrorKind::OutOfMemory));
    }

    let mut checker = Vec::new();
    for campaign in config.campaigns.values() {
        if checker.contains(&campaign.path.as_str()) {
            return Err(Error::from(ErrorKind::InvalidData));
        }

        checker.push(&campaign.path)
    }
    Ok(())
}
