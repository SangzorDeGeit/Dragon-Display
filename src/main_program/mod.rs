use gdk4::Monitor;
use gtk::prelude::GtkApplicationExt;

use crate::manage_campaign::config::CampaignData;
use crate::manage_campaign::gui::SYNCHRONIZATION_OPTIONS;
use crate::main_program::gui::select_monitor_window;

pub mod gui;

fn start_dragon_display(app: &adw::Application, campaign: &(String, CampaignData), display_monitor: &Monitor) {
    let monitor = select_monitor_window(&app, campaign);
    //checks if images need to be synchronized
    //starts the main application control panel
    //starts the main application image displayer
}

pub fn dragon_display_init(app: &adw::Application, campaign: &(String, CampaignData)) {
    let monitor = select_monitor_window(&app, campaign);
    //checks if images need to be synchronized
    //starts the main application control panel
}