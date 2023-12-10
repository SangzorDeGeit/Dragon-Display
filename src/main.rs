//GUI crates
use gtk::prelude::*;
use gtk::glib;
use manage_campaign_logic::CampaignData;
use tokio;

//imported modules
pub mod manage_campaign_gui;
pub mod google_drive_sync;
pub mod manage_campaign_logic;


pub const APP_ID: &str = "Dragon-Display";

#[tokio::main]
async fn main()-> glib::ExitCode {
    let app: adw::Application = adw::Application::builder().application_id(APP_ID).build();

    app.connect_activate(manage_campaign_gui::select_campaign_window);
    
    app.run()
}

fn run_program(campaign: &(String, CampaignData)){
    todo!()
}

