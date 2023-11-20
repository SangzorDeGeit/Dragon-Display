//GUI crates
use gtk::prelude::*;
use gtk::{glib, Application};
use manage_campaign_logic::CampaignData;

//imported modules
pub mod manage_campaign_gui;
pub mod google_drive_sync;
pub mod manage_campaign_logic;

use display_info::DisplayInfo;

const APP_ID: &str = "Dragon-Display";


fn main()-> glib::ExitCode {
    let app: adw::Application = adw::Application::builder().application_id(APP_ID).build();

    app.connect_activate(manage_campaign_gui::select_campaign_window);
    
    app.run()
}

fn run_program(campaign: &(String, CampaignData)){
    todo!()
}

fn get_monitor_info(){
    let display_infos = DisplayInfo::all().unwrap();
    for display_info in display_infos {
      println!("display_info {display_info:?}");
    } 
}
