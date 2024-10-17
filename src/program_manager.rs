use crate::{
    config::Campaign,
    ui::{control_window::ControlWindow, display_window::DdDisplayWindow},
};

use gdk4::Monitor;
use gtk::prelude::*;

pub fn dragon_display(app: &adw::Application, campaign: Campaign, monitor: Monitor) {
    let (sender, receiver) = async_channel::unbounded();

    let control_window = ControlWindow::new(&app, campaign, sender);
    control_window.present();
    control_window.maximize();
    let display_window = DdDisplayWindow::new(&app);
    display_window.present();
    display_window.fullscreen_on_monitor(&monitor);

    control_window.connect_destroy(move |_| {
        display_window.destroy();
    });
}
