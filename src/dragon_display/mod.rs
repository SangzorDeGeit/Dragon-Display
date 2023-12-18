use gdk4::Monitor;

use crate::manage_campaign::config::CampaignData;

use self::gui::select_screen_window;

pub mod gui;

fn start_dragon_display(app: &adw::Application, campaign: &(String, CampaignData), display_monitor: &Monitor) {
    let monitor = select_screen_window(&app, campaign);
    //checks if images need to be synchronized
    //starts the main application control panel
    //starts the main application image displayer
}