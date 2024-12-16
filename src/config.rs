// File containing functions that manage the config folder for campaign data
use crate::errors::ConfigError;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::remove_dir_all;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, ErrorKind, Read, Write},
};
use toml::to_string;

pub const IMAGE_EXTENSIONS: [&str; 6] = ["jpeg", "jpg", "png", "svg", "webp", "avif"];
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

impl Campaign {
    pub fn new(name: String, path: String) -> Self {
        Self {
            name,
            path,
            sync_option: SynchronizationOption::None,
        }
    }

    pub fn new_googledrive(
        name: String,
        path: String,
        access_token: String,
        refresh_token: String,
        google_drive_sync_folder: String,
    ) -> Self {
        Self {
            name,
            path,
            sync_option: SynchronizationOption::GoogleDrive {
                access_token,
                refresh_token,
                google_drive_sync_folder,
            },
        }
    }

    /// Updates the tokens returns a new instance of the campaign
    /// Can only be used for a googledrive campaign
    pub fn update_tokens(&self, access_token: String, refresh_token: String) -> Self {
        assert!(
            matches!(self.sync_option, SynchronizationOption::GoogleDrive { .. }),
            "Update tokens called for a non-googledrive-campaign"
        );
        match &self.sync_option {
            SynchronizationOption::None => self.to_owned(),
            SynchronizationOption::GoogleDrive {
                google_drive_sync_folder,
                ..
            } => Self {
                name: self.name.clone(),
                path: self.path.clone(),
                sync_option: SynchronizationOption::GoogleDrive {
                    access_token,
                    refresh_token,
                    google_drive_sync_folder: google_drive_sync_folder.to_string(),
                },
            },
        }
    }

    /// Returns all data of the campaign
    /// returns empty strings if the data is not applicable
    pub fn get_campaign_data(&self) -> (String, String, String, String, String) {
        match &self.sync_option {
            SynchronizationOption::None => (
                self.name.to_string(),
                self.path.to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
            ),
            SynchronizationOption::GoogleDrive {
                access_token,
                refresh_token,
                google_drive_sync_folder,
            } => (
                self.name.to_string(),
                self.path.to_string(),
                access_token.to_string(),
                refresh_token.to_string(),
                google_drive_sync_folder.to_string(),
            ),
        }
    }
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

impl Default for SynchronizationOption {
    fn default() -> Self {
        SynchronizationOption::None
    }
}

/// Tries to read the campaign configurations from the config file and puts them in a Vector.
/// if there is no config file this method will return an empty vector
pub fn read_campaign_from_config() -> Result<Vec<Campaign>> {
    // We should return an empty vector if the config file is not found
    let mut file = match get_campaign_config(Operation::READ) {
        Ok(f) => f,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => return Ok(Vec::new()),
            _ => bail!(e.kind()),
        },
    };

    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {}
        Err(_) => bail!(ConfigError::InvalidConfig),
    };

    let config: Config = match toml::from_str(&contents) {
        Ok(d) => d,
        Err(_) => bail!(ConfigError::InvalidConfig),
    };

    check_integrity(&config)?;

    Ok(config.campaigns)
}

/// Given a hashmap with the campaign name as key and corresponding campaigndata as value, this function will try to write the campaign to the config file and create a directory in the campaign.path. This function will update the values if the name of the campaign already exists
pub fn write_campaign_to_config(campaign: Campaign) -> Result<()> {
    let config_item = Config {
        campaigns: vec![campaign.clone()],
    };
    // if it exists we want to remove it to add the updated version
    if campaign_exists(&campaign)? {
        remove_campaign_from_config(campaign.clone(), false)?;
    }
    let mut config_file = get_campaign_config(Operation::APPEND)?;
    let toml_string = to_string(&config_item).unwrap();
    config_file.write_all(toml_string.as_bytes())?;
    fs::create_dir_all(campaign.path)?;
    Ok(())
}

/// Given an existing campaign name this function will remove this campaign and all the campaigndata from the config file.
pub fn remove_campaign_from_config(campaign: Campaign, remove_folder: bool) -> Result<()> {
    check_save_removal(&campaign.path)?;
    let campaign_list = read_campaign_from_config()?;

    if campaign_list.len() == 0 {
        bail!(ConfigError::CampaignNotFound);
    }
    if campaign_list.len() == 1 {
        if remove_folder {
            remove_dir_all(&campaign.path).unwrap_or(());
        }
        return remove_campaign_config();
    }

    let mut new_campaign_list = Vec::from(campaign_list);

    let index = match new_campaign_list
        .iter()
        .position(|c| c.name == campaign.name)
    {
        Some(i) => i,
        None => bail!(ConfigError::CampaignNotFound),
    };

    new_campaign_list.swap_remove(index);

    let config_item = Config {
        campaigns: new_campaign_list,
    };
    let mut config_file = get_campaign_config(Operation::WRITE)?;
    let toml_string = to_string(&config_item).unwrap();
    config_file.write_all(toml_string.as_bytes())?;
    if remove_folder {
        remove_dir_all(&campaign.path).unwrap_or(());
    }
    Ok(())
}

/// Given a campaign, this function will return whether this campaign exists in the config file
fn campaign_exists(campaign: &Campaign) -> Result<bool> {
    let campaign_list = read_campaign_from_config()?;
    for c in campaign_list {
        if c.name == campaign.name {
            return Ok(true);
        }
    }
    Ok(false)
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
fn remove_campaign_config() -> Result<()> {
    let mut path = env::current_dir()?;
    path.push(".config.toml");
    fs::remove_file(&path)?;
    Ok(())
}

/// Checks for the integrity of the config file.  
/// Checks if there are no more campaigns in the file than MAX_CAMPAIGN_AMOUNT  
/// Checks if there are no duplicate paths in the campaign folder
fn check_integrity(config: &Config) -> Result<()> {
    if config.campaigns.len() > usize::from(MAX_CAMPAIGN_AMOUNT) {
        bail!(ConfigError::TooManyCampaigns);
    }

    let mut path_names = Vec::new();
    for campaign in &config.campaigns {
        let path = &campaign.path;
        if path_names.contains(&path) {
            bail!(ConfigError::DuplicateCampaign);
        }
        path_names.push(path);
    }

    Ok(())
}

/// Check if there only image files in the folder of the campaign to be removed
fn check_save_removal(campaign_path: &str) -> Result<()> {
    let files = match fs::read_dir(&campaign_path) {
        Ok(f) => f,
        Err(e) => match e.kind() {
            ErrorKind::NotFound => return Ok(()),
            ErrorKind::PermissionDenied => bail!(ConfigError::PermissionDenied),
            _ => bail!(ConfigError::Other),
        },
    };

    for file in files {
        let file_path = file?.path();

        let extension_os = file_path
            .extension()
            .context("Could not get file extensions")?;

        let extension = extension_os
            .to_str()
            .context("Could not convert file types into strings")?;

        if !IMAGE_EXTENSIONS.contains(&extension) {
            bail!(ConfigError::CouldNotRemove);
        }
    }

    Ok(())
}
