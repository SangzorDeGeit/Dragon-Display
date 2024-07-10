//GUI crates
use gtk::prelude::*;
use gtk::glib;

//imported modules
pub mod dragon_display;
pub mod widgets;


use dragon_display::setup::select_campaign;


pub const APP_ID: &str = "display.dragon";

#[tokio::main]
async fn main()-> glib::ExitCode {
    let app: adw::Application = adw::Application::builder().application_id(APP_ID).build();

    app.connect_activate(select_campaign);        
    
    app.run()

}


