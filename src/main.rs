use std::collections::HashMap;

//GUI crates
use gtk::prelude::*;
use gtk::{glib, Application};
use manage_campaign_logic::CampaignData;

//imported modules
pub mod manage_campaign_gui;
pub mod google_drive_sync;
pub mod manage_campaign_logic;
pub mod toml_test;


const APP_ID: &str = "Dragon-Display";

fn main() -> glib::ExitCode {
    let app: Application = Application::builder().application_id(APP_ID).build();

    app.connect_activate(manage_campaign_gui::select_campaign_window);
    
    app.run()
}

fn run_program(campaign: &(String, CampaignData)){
    todo!()
}
