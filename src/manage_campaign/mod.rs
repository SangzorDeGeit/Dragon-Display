pub mod gui;
pub mod config;

use std::{env, fs, io, io::{Error, ErrorKind}};
use std::collections::HashMap;

use config::{CampaignData, write_campaign_to_config, remove_campaign_from_config};
use gui::{create_error_dialog, select_campaign_window};


const IMAGE_EXTENSIONS: [&str; 6] = ["jpeg", "jpg", "png", "svg", "webp", "avif"];

//TODO: MAKE FOLLOWING TWO FUNCTIONS INTO ONE FUNCTION, CURRENTLY TWO DIFFERENT FUNCTIONS SINCE NOT SURE WHAT GOOGLE DRIVE NEEDS
pub fn add_gd_campaign(app: &adw::Application, path: &str, access_token: &str, sync_option: &str) {
    //try to make the folder given by 'path', if it exists, continue
    let campaign_values = CampaignData {
        sync_option: sync_option.to_string(),
        path : path.to_string(),
        access_token: Some(access_token.to_string())
    };

    add_campaign(&app, campaign_values)
}


pub fn add_none_campaign(app: &adw::Application, path: &str, sync_option: &str) {
     //try to make the folder given by 'path', if it exists, continue
    let campaign_values = CampaignData {
        sync_option: sync_option.to_string(),
        path : path.to_string(),
        access_token: None
    };

    add_campaign(&app, campaign_values)

}



// This function is called by the gui modules to create the campaign
fn add_campaign(app: &adw::Application, campaign_values: CampaignData){
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




// This function is called by the gui modules to remove given campaign
// TODO: any envirnoment variables for sync services should be removed
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

fn check_save_removal(campaign_path: &str) -> Result<(), io::Error> {
    let files = fs::read_dir(&campaign_path)?;
    for file in files {
        let file = file?;
        match file.path().extension() {
            Some(extension) => {
                match extension.to_str() {
                    Some(ext) => {
                        if !IMAGE_EXTENSIONS.contains(&ext) {
                            return Err(Error::from(ErrorKind::WouldBlock))
                        }
                    },
                    None => return Err(Error::from(ErrorKind::NotFound))
                }
            }
            None => return Err(Error::from(ErrorKind::NotFound))
        }
    }
    Ok(())
}