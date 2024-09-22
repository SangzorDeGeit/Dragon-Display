// File containing functions that manage the config folder for campaign data
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::remove_dir_all;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, Error, ErrorKind, Read, Write},
};
use toml::to_string;

const IMAGE_EXTENSIONS: [&str; 6] = ["jpeg", "jpg", "png", "svg", "webp", "avif"];
pub const CAMPAIGN_MAX_CHAR_LENGTH: u16 = 25;
pub const MAX_CAMPAIGN_AMOUNT: u16 = 10;
pub const SYNCHRONIZATION_OPTIONS: [&str; 2] = ["None", "Google Drive"];

enum Operation {
    READ,
    WRITE,
    APPEND,
}

/// Structure representing the name of the campaign and the corresponding data
#[derive(Serialize, Deserialize, Default)]
struct Config {
    campaigns: Vec<Campaign>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Campaign {
    pub name: String,
    pub path: String,
    pub sync_option: SynchronizationOption,
}

impl Default for Campaign {
    fn default() -> Self {
        Campaign {
            name: "".to_string(),
            path: "".to_string(),
            sync_option: SynchronizationOption::None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SynchronizationOption {
    None,
    GoogleDrive {
        access_token: String,
        refresh_token: String,
        google_drive_sync_folder: String,
    },
}

/// Tries to read the campaign configurations from the config file and puts them in a Vector.
/// if there is no config file this method will return an empty vector
pub fn read_campaign_from_config() -> Result<Vec<Campaign>, Error> {
    // We should return an empty vector if the config file is not found
    let mut file = match get_campaign_config(Operation::READ) {
        Ok(f) => f,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => return Ok(Vec::new()),
            _ => return Err(e),
        },
    };

    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {}
        Err(_) => return Err(Error::new(ErrorKind::InvalidData, "The config folder did not contain valid UTF-8, remove the .config.toml file, which is a hidden file in the current directory and restart the program")),
    };

    let config: Config = match toml::from_str(&contents) {
        Ok(d) => d,
        Err(_) => return Err(Error::new(ErrorKind::InvalidData, "Could not read campaigns from the config file, remove the .config.toml file, which is a hidden file in the current directory and restart the program")),
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
    check_save_removal(&campaign.path)?;
    let campaign_list = read_campaign_from_config()?;

    if campaign_list.len() == 0 {
        return Err(Error::new(
            ErrorKind::NotFound,
            "Expected a config folder but could not find one while deleting the campaign",
        ));
    }
    if campaign_list.len() == 1 {
        remove_dir_all(&campaign.path).unwrap_or(());
        return remove_campaign_config();
    }

    let mut new_campaign_list = Vec::from(campaign_list);

    let index = match new_campaign_list
        .iter()
        .position(|c| c.name == campaign.name)
    {
        Some(i) => i,
        None => {
            return Err(Error::new(
                ErrorKind::NotFound,
                "Could not find the campaign to be deleted in the config folder",
            ))
        }
    };

    new_campaign_list.swap_remove(index);

    let config_item = Config {
        campaigns: new_campaign_list,
    };
    let mut config_file = get_campaign_config(Operation::WRITE)?;
    let toml_string = to_string(&config_item).unwrap();
    config_file.write_all(toml_string.as_bytes())?;
    remove_dir_all(&campaign.path).unwrap_or(());
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
        return Err(Error::new(ErrorKind::OutOfMemory, "There were more campaigns found in the config file then allowed, remove the config file which is a hidden file in the current directory and restart the program"));
    }

    let mut path_names = Vec::new();
    for campaign in &config.campaigns {
        let path = &campaign.path;
        if path_names.contains(&path) {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Multiple campaigns were found with the same image folder path",
            ));
        }
        path_names.push(path);
    }

    Ok(())
}

/// Check if there only image files in the folder of the campaign to be removed
fn check_save_removal(campaign_path: &str) -> Result<(), io::Error> {
    let files = match fs::read_dir(&campaign_path) {
        Ok(f) => f,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => return Ok(()),
            ErrorKind::PermissionDenied => {
                return Err(Error::new(
                    ErrorKind::PermissionDenied,
                    "Insufficient permissions to find the image folder of the campaign",
                ))
            }
            _ => return Err(Error::new(
                ErrorKind::Other,
                "An unexpected error occured while trying to find the image folder of the campaign",
            )),
        },
    };

    for file in files {
        let file_path = file?.path();

        let extension_os = match file_path.extension() {
            Some(e) => e,
            None => return Err(Error::new(ErrorKind::NotFound, "Could not remove image folder: failed to read the file types in the campaign folder")),
        };

        let extension = match extension_os.to_str() {
            Some(e) => e,
            None => {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    "Could not convert file types into strings",
                ))
            }
        };

        if !IMAGE_EXTENSIONS.contains(&extension) {
            return Err(Error::new(
                ErrorKind::NotFound,
                "Could not remove image folder: found non-image files in the campaign image folder",
            ));
        }
    }

    Ok(())
}
