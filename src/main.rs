use std::sync::OnceLock;

//GUI crates
use gtk::prelude::*;
use gtk::glib;

//imported modules
pub mod dragon_display;
pub mod widgets;


use dragon_display::setup::select_campaign;
use tokio::runtime::Runtime;


pub const APP_ID: &str = "display.dragon";
 
pub fn runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        Runtime::new().expect("The tokio runtime setup needs to complete")
    })
}

fn main()-> glib::ExitCode {
    let app: adw::Application = adw::Application::builder().application_id(APP_ID).build();

    app.connect_activate(select_campaign);        
    
    app.run()

}


