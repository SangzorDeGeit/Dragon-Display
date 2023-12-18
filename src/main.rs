//GUI crates
use gtk::prelude::*;
use gtk::glib;

//core crates
use tokio;

//imported modules
pub mod google_drive_sync;
pub mod manage_campaign;
pub mod dragon_display;

use manage_campaign::gui::select_campaign_window;


pub const APP_ID: &str = "Dragon-Display";

#[tokio::main]
async fn main()-> glib::ExitCode {
    let app: adw::Application = adw::Application::builder().application_id(APP_ID).build();

    app.connect_activate(select_campaign_window);
    
    app.run()
}


