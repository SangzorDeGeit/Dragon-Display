//Packages for configuration file
use std::fs::File;
use std::io::{Read, ErrorKind, Write};
use serde::{Deserialize, Serialize};
use toml::to_string;

//GUI crates
use gtk::prelude::*;
use gtk::{glib, Application};

//imported modules
pub mod campaigns;
pub mod google_drive_sync;


const APP_ID: &str = "Dragon-Display";

fn main() -> glib::ExitCode {
    let app: Application = Application::builder().application_id(APP_ID).build();

    app.connect_activate(campaigns::select_campaign_window);

    app.run()
}

