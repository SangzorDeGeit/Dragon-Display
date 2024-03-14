use std::{fs::{File, OpenOptions, self}, io::{Write, self, Error, ErrorKind, Read}};
use serde::{Deserialize, Serialize};
use toml::to_string;
use std::env;
use std::collections::HashMap;

use crate::manage_campaign::gui::MAX_CAMPAIGN_AMOUNT;

const CONFIG_OPERATION_READ : u8 = 0;
const CONFIG_OPERATION_APPEND : u8 = 1;
const CONFIG_OPERATION_WRITE : u8 = 2;


#[derive(Serialize, Deserialize)]
struct Config {
    campaigns : HashMap<String, CampaignData>
}

#[derive(Serialize, Deserialize)]
pub struct CampaignData {
    pub sync_option : String,
    pub path : String,
    pub access_token : Option<String>,
    pub refresh_token : Option<String>
}



pub fn read_campaign_from_config() -> Result<HashMap<String, CampaignData>, Error> {
    let mut file = get_campaign_config(CONFIG_OPERATION_READ)?;

    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(_) => return Err(Error::from(ErrorKind::Unsupported))
    };

    let config: Config = match toml::from_str(&contents) {
        Ok(d) => d,
        Err(_) => return Err(Error::from(ErrorKind::InvalidData))
    };

    check_integrity(&config)?;

    Ok(config.campaigns)
}



pub fn write_campaign_to_config(campaign: HashMap<String, CampaignData>) -> Result<(), io::Error>{
    let config_item = Config{campaigns: campaign};
    let mut config_file = get_campaign_config(CONFIG_OPERATION_APPEND)?;
    let toml_string = to_string(&config_item).unwrap();
    config_file.write_all(toml_string.as_bytes())?;
    Ok(())
}



pub fn remove_campaign_from_config(campaign_name: &str) -> Result<(), io::Error> {
    let campaign_list = read_campaign_from_config()?;

    let mut new_campaign_list = HashMap::new();

    if campaign_list.len() > 1 {
        for campaign in campaign_list {
            if campaign.0.as_str() != campaign_name {
                new_campaign_list.insert(campaign.0, campaign.1);
            }
        }


        let config_item = Config{campaigns: new_campaign_list};
        let mut config_file = get_campaign_config(CONFIG_OPERATION_WRITE)?;
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



fn get_campaign_config(operation: u8) -> Result<File, io::Error>{
    let mut path = env::current_dir()?;
    path.push(".config.toml");
    match operation {
        CONFIG_OPERATION_READ => {
            let file = OpenOptions::new()
                .read(true)
                .open(&path)?;
            Ok(file)
        },
        CONFIG_OPERATION_WRITE => {
            let file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&path)?;
            Ok(file)
        },
        CONFIG_OPERATION_APPEND => {
            let file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(&path)?;
            Ok(file)
        },
        _ => Err(Error::from(ErrorKind::InvalidInput))
    }
}



fn remove_campaign_config() -> Result<(), io::Error> {
    let mut path = env::current_dir()?;
    path.push(".config.toml");
    fs::remove_file(&path)?;
    Ok(())
}


fn check_integrity(config: &Config) -> Result<(), io::Error> {
    //Check if no campaigns are manually edited via the toml file to exceed the maximum number of campaigns
    if config.campaigns.len() > usize::from(MAX_CAMPAIGN_AMOUNT) {
        return Err(Error::from(ErrorKind::OutOfMemory))
    }

    //Check if no two campaigns have the same campaign path
    let mut checker = Vec::new();
    for campaign in config.campaigns.values(){
        if checker.contains(&campaign.path.as_str()) {
            return Err(Error::from(ErrorKind::InvalidData))
        }
        
        checker.push(&campaign.path)
    }
    Ok(())
}