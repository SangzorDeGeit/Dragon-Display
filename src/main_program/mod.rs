use gdk4::Monitor;
use gtk::prelude::GtkApplicationExt;

use crate::manage_campaign::config::CampaignData;
use crate::manage_campaign::gui::SYNCHRONIZATION_OPTIONS;
use crate::main_program::gui::select_monitor_window;
use crate::google_drive_sync::sync_drive;

pub mod gui;

fn start_dragon_display(app: &adw::Application, campaign: &(String, CampaignData), display_monitor: &Monitor) {
    let monitor = select_monitor_window(&app, campaign);
    //checks if images need to be synchronized
    //starts the main application control panel
    //starts the main application image displayer
}

pub fn dragon_display_init(app: &adw::Application, campaign: &(String, CampaignData)) {
    if campaign.1.sync_option==SYNCHRONIZATION_OPTIONS[1] {
        let access_token = match &campaign.1.access_token {
            Some(t) => t,
            None => panic!("Google drive sync option but no access token")
        };
        let refresh_token = match &campaign.1.refresh_token {
            Some(t) => t,
            None => panic!("Google drive sync option but no refresh token")
        };

        sync_drive(&access_token, &refresh_token);
    }
    let monitor = select_monitor_window(&app, campaign);
    //checks if images need to be synchronized
    //starts the main application control panel
}