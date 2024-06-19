use std::io::Error;

use adw::prelude::*;
use gtk::{glib, ApplicationWindow, Button, Grid, Label};
use manage_campaign::config::{remove_campaign_from_config, write_campaign_to_config, Campaign};

use crate::dragon_display::gui::select_monitor_window;
use crate::dragon_display::manage_campaign::gui::{
    add_campaign_window, remove_campaign_window, select_campaign_window,
};

pub mod google_drive_sync;
pub mod gui;
pub mod manage_campaign;

/// The messages that the select_campaign_window can send
pub enum SelectMessage {
    Campaign { campaign: Campaign },
    Remove,
    Add,
    Error { error: Error, fatal: bool },
}

/// The messages that the remove and add campaign window can send
pub enum AddRemoveMessage {
    Campaign { campaign: Campaign },
    Cancel,
    Error { error: Error, fatal: bool },
}

pub fn select_campaign(app: &adw::Application) {
    let (sender, receiver) = async_channel::bounded(1);

    let window = match select_campaign_window(app, sender) {
        Ok(w) => w,
        Err(error) => {
            handle_manage_campaign_error(app, error, true);
            return;
        }
    };
    window.present();

    // We have to await messages from the channel without blocking the main event loop
    glib::spawn_future_local(glib::clone!( @weak window, @weak app => async move {
        while let Ok(message) = receiver.recv().await {
            match message {
                SelectMessage::Campaign { campaign } => {
                    window.destroy();
                    start_dragon_display(&app, campaign);
                }
                SelectMessage::Remove => {
                    window.destroy();
                    remove_campaign(&app);
                },
                SelectMessage::Add => {
                    window.destroy();
                    add_campaign(&app);
                },
                SelectMessage::Error { error, fatal } => handle_manage_campaign_error(&app, error, fatal),
            }
        }
    }));
}

fn remove_campaign(app: &adw::Application) {
    let (sender, receiver) = async_channel::bounded(1);
    let window = match remove_campaign_window(app, sender) {
        Ok(w) => w,
        Err(error) => {
            handle_manage_campaign_error(app, error, true);
            return;
        }
    };
    window.present();

    // We have to await messages from the channel without blocking the main event loop
    glib::spawn_future_local(glib::clone!( @weak window, @weak app => async move {
        while let Ok(message) = receiver.recv().await {
            match message {
                AddRemoveMessage::Campaign { campaign } => {
                    window.destroy();
                    match remove_campaign_from_config(campaign) {
                        Ok(_) => (),
                        Err(e) => handle_manage_campaign_error(&app, e, false),
                    }
                    remove_campaign(&app);
                }
                AddRemoveMessage::Cancel => {
                    window.destroy();
                    select_campaign(&app);
                },
                AddRemoveMessage::Error { error, fatal } => handle_manage_campaign_error(&app, error, fatal),
            }
        }
    }));
}

fn add_campaign(app: &adw::Application) {
    let (sender, receiver) = async_channel::bounded(1);
    let window = match add_campaign_window(app, sender) {
        Ok(w) => w,
        Err(error) => {
            handle_manage_campaign_error(app, error, true);
            return;
        }
    };
    window.present();

    // We have to await messages from the channel without blocking the main event loop
    glib::spawn_future_local(glib::clone!( @weak window, @weak app => async move {
        while let Ok(message) = receiver.recv().await {
            match message {
                AddRemoveMessage::Campaign { campaign } => {
                    window.destroy();
                    let _ = write_campaign_to_config(campaign);
                    select_campaign(&app);
                }
                AddRemoveMessage::Cancel => {
                    window.destroy();
                    select_campaign(&app);
                },
                AddRemoveMessage::Error { error, fatal } => handle_manage_campaign_error(&app, error, fatal),
            }
        }
    }));
}

pub fn start_dragon_display(app: &adw::Application, campaign: Campaign) {
    let monitor = select_monitor_window(&app, campaign);
    //checks if images need to be synchronized
    //starts the main application control panel
    //starts the main application image displayer
}

/// Function that produces proper error messages (dialogs) based on errors that are given
pub fn handle_manage_campaign_error(app: &adw::Application, error: Error, fatal: bool) {
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
