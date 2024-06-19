//GUI crates
use gtk::prelude::*;
use gtk::glib;

//core crates
use tokio;

//imported modules
pub mod dragon_display;
pub mod widgets;


use dragon_display::select_campaign;


pub const APP_ID: &str = "Dragon-Display";

#[tokio::main]
async fn main()-> glib::ExitCode {
    let app: adw::Application = adw::Application::builder().application_id(APP_ID).build();

    app.connect_activate(select_campaign);        
    
    app.run()

}


