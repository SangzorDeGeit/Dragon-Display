use gdk4::Monitor;
use gtk::prelude::GtkApplicationExt;
use manage_campaign::config::Campaign;

use crate::dragon_display::gui::select_monitor_window;
use crate::dragon_display::google_drive_sync::sync_drive;
use crate::dragon_display::manage_campaign::SYNCHRONIZATION_OPTIONS;

pub mod gui;
pub mod manage_campaign;
pub mod google_drive_sync;



pub fn program(app: &adw::Application) {

}


pub fn start_dragon_display(app: &adw::Application, campaign: Box<Campaign>) {
    let monitor = select_monitor_window(&app, campaign);
    //checks if images need to be synchronized
    //starts the main application control panel
    //starts the main application image displayer
}
