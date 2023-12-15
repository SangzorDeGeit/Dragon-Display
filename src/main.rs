//GUI crates
use gtk::prelude::*;
use gtk::glib;
use tokio;

//imported modules
pub mod google_drive_sync;
pub mod manage_campaign;

use manage_campaign::gui::select_campaign_window;
use manage_campaign::config::CampaignData;



pub const APP_ID: &str = "Dragon-Display";

#[tokio::main]
async fn main()-> glib::ExitCode {
    let app: adw::Application = adw::Application::builder().application_id(APP_ID).build();

    app.connect_activate(select_campaign_window);
    
    app.run()
}

fn run_program(campaign: &(String, CampaignData)){
    todo!()
}

fn open_window() {
    let display = gdk4::Display::default();
    match display {
        Some(d) => {
            let monitor = d.monitors().item(1);
            match monitor {
                Some(m) => {
                    let mon = m.to_value().get::<gdk4::Monitor>().expect("The value needs to be monitor!");
                    // window.fullscreen_on_monitor(&mon);
                },
                None => {}
            }
            
        },
        None => {}
    }
}

