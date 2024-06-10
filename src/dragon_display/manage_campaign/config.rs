use gtk::glib::{property, HasParamSpec};
use serde::{Deserialize, Serialize};
use std::env;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Error, ErrorKind, Read, Write},
};
use toml::to_string;

use crate::dragon_display::manage_campaign::MAX_CAMPAIGN_AMOUNT;

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
#[derive(Serialize, Deserialize, Default)]
struct Config {
    campaigns: Vec<Campaign>
}

#[derive(Serialize, Deserialize)]
pub struct Campaign {
    pub name: String,
    pub sync_option: SynchronizationOption,
}

impl Default for Campaign {
    fn default() -> Self {
        Campaign { name: "".to_string(), 
                sync_option: SynchronizationOption::None { path: "".to_string() } }
    }
}

#[derive(Serialize, Deserialize)]
pub enum SynchronizationOption {
    None {path: String},
    GoogleDrive {path: String,
                access_token: String,
                refresh_token: String},
}

/// Tries to read the campaign configurations from the config file and puts them in a hashmap.
pub fn read_campaign_from_config() -> Result<Vec<Campaign>, Error> {
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
pub fn write_campaign_to_config(campaign: Campaign) -> Result<(), io::Error> {
    let config_item = Config {
        campaigns: vec![campaign],
    };

    let mut config_file = get_campaign_config(Operation::APPEND)?;
    let toml_string = to_string(&config_item).unwrap();
    config_file.write_all(toml_string.as_bytes())?;
    Ok(())
}

/// Given an existing campaign name this function will remove this campaign and all the campaigndata from the config file.
pub fn remove_campaign_from_config(campaign: Campaign) -> Result<(), io::Error> {
    let campaign_list = read_campaign_from_config()?;

    if campaign_list.len() == 0 {
        return Err(Error::from(ErrorKind::NotFound))
    }
    if campaign_list.len() == 1 {
        return remove_campaign_config();
    }

    let mut new_campaign_list = Vec::from(campaign_list);

    let index = match new_campaign_list.iter().position(|&c| c.name == campaign.name) {
        Some(i) => i,
        None => return Err(Error::from(ErrorKind::NotFound)),
    };

    new_campaign_list.swap_remove(index);

    let config_item = Config {
        campaigns: new_campaign_list,
    };
    let mut config_file = get_campaign_config(Operation::WRITE)?;
    let toml_string = to_string(&config_item).unwrap();
    config_file.write_all(toml_string.as_bytes())?;
    Ok(())
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

/// Checks for the integrity of the config file.  
/// Checks if there are no more campaigns in the file than MAX_CAMPAIGN_AMOUNT  
/// Checks if there are no duplicate paths in the campaign folder
fn check_integrity(config: &Config) -> Result<(), io::Error> {
    if config.campaigns.len() > usize::from(MAX_CAMPAIGN_AMOUNT) {
        return Err(Error::from(ErrorKind::OutOfMemory));
    }

    let mut path_vector = Vec::new();
    for campaign in config.campaigns {
        let path = match campaign.sync_option {
            SynchronizationOption::None { path } => path,
            SynchronizationOption::GoogleDrive { path, access_token, refresh_token } => path,
        };
        if path_vector.contains(&path) {
            return Err(Error::from(ErrorKind::InvalidData));
        }
        path_vector.push(path);
    }

    Ok(())
}
