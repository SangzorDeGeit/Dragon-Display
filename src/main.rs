use std::sync::OnceLock;

//GUI crates
use gtk::prelude::*;
use gtk::{gio, glib};

//imported modules
pub mod campaign;
pub mod config;
pub mod errors;
pub mod gd_client;
pub mod google_drive;
pub mod program_manager;
pub mod setup;
pub mod setup_manager;
pub mod ui;
pub mod widgets;

use setup_manager::select_campaign;
use tokio::runtime::Runtime;

pub const APP_ID: &str = "com.github.SangzorDeGeit.Dragon-Display";

#[macro_export]
/// Extract the Ok value of a result or intterupt the process emitting an error signal using the
/// local 'emit_error() function for the self object'. Supply a boolean value to indicate whether
/// the error is (true:) a fatal error (should close the program) or (false:) not.
macro_rules! try_emit {
    ($self:ident, $result: expr, $fatal: ident) => {
        match $result {
            Ok(val) => val,
            Err(err) => {
                $self.emit_error(err, $fatal);
                return;
            }
        }
    };
}

pub fn runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| Runtime::new().expect("The tokio runtime setup needs to complete"))
}

fn main() -> glib::ExitCode {
    //register resources
    gio::resources_register_include!("dragon_display.gresource")
        .expect("Failed to register resources");
    let app: adw::Application = adw::Application::builder().application_id(APP_ID).build();

    app.connect_activate(select_campaign);

    app.run()
}
