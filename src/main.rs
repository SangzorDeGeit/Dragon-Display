//GUI crates
use gtk::prelude::*;
use gtk::{glib, Application};

//imported modules
pub mod manage_campaign_gui;
pub mod google_drive_sync;
pub mod manage_campaign_logic;


const APP_ID: &str = "Dragon-Display";

fn main() -> glib::ExitCode {
    let app: Application = Application::builder().application_id(APP_ID).build();

    app.connect_activate(manage_campaign_gui::select_campaign_window);

    app.run()
}

