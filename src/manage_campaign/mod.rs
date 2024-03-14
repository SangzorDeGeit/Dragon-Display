pub mod gui;
pub mod config;

use std::{env, fs, io, io::{Error, ErrorKind}};
use std::collections::HashMap;

use config::{CampaignData, write_campaign_to_config, remove_campaign_from_config};
use gui::{create_error_dialog, select_campaign_window};


const IMAGE_EXTENSIONS: [&str; 6] = ["jpeg", "jpg", "png", "svg", "webp", "avif"];

/**
 * Add a campaign and write data to .config.toml
 */
pub fn add_campaign(app: &adw::Application, path: &str, access_token: Option<String>, refresh_token: Option<String>, sync_option: &str) {
    //try to make the folder given by 'path', if it exists, continue
    let campaign_values = CampaignData {
        sync_option: sync_option.to_string(),
        path : path.to_string(),
        access_token: access_token,
        refresh_token: refresh_token
    };

    let name = match env::var("CAMPAIGN_NAME") {
        Ok(n) => n,
        Err(_) => {
            create_error_dialog(app, "Could not find a campaign name");
            select_campaign_window(app);
            return;
        }    
    };


    let mut campaign = HashMap::new();
    campaign.insert(name.to_string(), campaign_values);

    match write_campaign_to_config(campaign) {
        Ok(_) => select_campaign_window(app),
        Err(error) => {
            let msg = format!("Could not add campaign: {}", error.to_string());
            create_error_dialog(app, &msg.as_str());
            select_campaign_window(app)
        }
    } 
}


/**
 * This function is called by the gui modules to remove given campaign
 * TODO: any envirnoment variables for sync services should be removed -> these are probably not set
 */ 
pub fn remove_campaign(app: &adw::Application, campaign_name: &str, campaign_path: &str) {
    match check_save_removal(&campaign_path) {
        Ok(_) => {
            match fs::remove_dir_all(&campaign_path) {
                Ok(_) => {},
                Err(_) => create_error_dialog(&app, "could not delete the campaign image folder")
            }
        },
        Err(_) => create_error_dialog(&app, "Did not remove the campaign image folder since non-image files were found in this directory")
    }

    match remove_campaign_from_config(campaign_name) {
        Ok(_) => select_campaign_window(app),
        Err(error) => {
            let msg = format!("Could not remove campaign: {}", error.to_string());
            create_error_dialog(app, &msg.as_str());
            select_campaign_window(app)
        }
    }
}

/**
 * Checks if there are only image files in the image folder of the campaign to be removed
 */
fn check_save_removal(campaign_path: &str) -> Result<(), io::Error> {
    let files = fs::read_dir(&campaign_path)?;
    for file in files {
        let file_path = file?.path();

        let extension_os = match file_path.extension() {
            Some(e) => e,
            None => return Err(Error::from(ErrorKind::NotFound))
        };

        let extension = match extension_os.to_str() {
            Some(e) => e,
            None => return Err(Error::from(ErrorKind::NotFound))
        };

        if !IMAGE_EXTENSIONS.contains(&extension) {
            return Err(Error::from(ErrorKind::WouldBlock))
        }

    }
        
    Ok(())
}