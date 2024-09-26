use gtk::prelude::*;
use gtk::{glib, ApplicationWindow, Button, Grid, Label};

use std::io::Error;

pub fn handle_setup_error(app: &adw::Application, error: Error, fatal: bool) {
    let msg = error.to_string();

    let container = Grid::new();
    let window = ApplicationWindow::builder()
        .application(app)
        .modal(true)
        .deletable(false)
        .child(&container)
        .build();

    if fatal {
        window.set_title(Some("Dragon-Display fatal error!"));
    } else {
        window.set_title(Some("Dragon-Display error!"));
    }

    let label = Label::builder()
        .label(&msg)
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();
    let button_ok = Button::builder()
        .label("Ok")
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();

    container.attach(&label, 0, 0, 1, 1);
    container.attach(&button_ok, 0, 1, 1, 1);

    button_ok.connect_clicked(glib::clone!(@strong app, @strong window => move |_| {
        window.destroy();
        if fatal {
            app.quit();
        }
    }));

    window.present();
}
