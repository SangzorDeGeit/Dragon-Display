pub mod ui;

use gdk4::Monitor;
use gtk::prelude::*;

use crate::dragon_display::setup::config::Campaign;
use ui::control_panel::control_panel_window;
use ui::display::display_window;

pub enum Message {
    Image { picture_path: String },
}

pub fn start_dragon_display(app: &adw::Application, campaign: Campaign, display_monitor: Monitor) {
    let (display_sender, display_receiver) = async_channel::unbounded();
    // start the display window
    let display_window = display_window(&app, display_receiver);
    display_window.present();
    display_window.fullscreen_on_monitor(&display_monitor);

    let control_window = match control_panel_window(app, campaign, display_sender) {
        Ok(w) => w,
        Err(_) => panic!("Error occured during control panel creation"),
    };
    control_window.present();

    control_window.connect_destroy(move |_| {
        display_window.destroy();
    });
}
