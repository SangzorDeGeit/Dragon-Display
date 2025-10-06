use std::sync::OnceLock;

use gtk::glib::clone;
//GUI crates
use gtk::prelude::*;
use gtk::{gio, glib};

//imported modules
pub mod campaign;
pub mod config;
pub mod errors;
pub mod gd_client;
pub mod program;
pub mod setup;
pub mod ui;
pub mod widgets;

use setup::DragonDisplaySetup;
use tokio::runtime::Runtime;
use ui::error_dialog::ErrorDialog;

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

    let setup = DragonDisplaySetup::new();

    app.connect_activate(clone!(@weak setup => move |app| {
        setup.select_window(&app);
    }));

    setup.connect_error(clone!(@weak app => move |_, msg, fatal| {
        ErrorDialog::new(&app, msg, fatal).present();
    }));

    app.run()
}
