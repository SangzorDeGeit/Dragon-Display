pub mod config;
pub mod google_drive;
pub mod gui;
pub mod helper;
pub mod ui;

use std::io::Error;

use adw::prelude::*;
use config::{remove_campaign_from_config, write_campaign_to_config};
use glib::clone;
use gtk::{glib, ApplicationWindow, Button, Grid, Label};

use config::{Campaign, SynchronizationOption};

use ui::add_campaign::add_campaign_window;
use ui::google_drive::{googledrive_connect_window, googledrive_select_path_window};
use ui::remove_campaign::remove_campaign_window;
use ui::select_campaign::select_campaign_window;

/// The messages that the select_campaign_window can send
pub enum SelectMessage {
    Campaign { campaign: Campaign },
    Remove,
    Add,
    Error { error: Error, fatal: bool },
}

/// The messages that the add and remove campaign window can send
pub enum AddRemoveMessage {
    Campaign { campaign: Campaign },
    Cancel,
    Error { error: Error, fatal: bool },
}

/// Make and present the window to select a campaign
pub fn select_campaign(app: &adw::Application) {
    // This channel lets the frontend(ui) communicate with the manager
    let (sender, receiver) = async_channel::bounded(1);

    let window = match select_campaign_window(app, sender) {
        Ok(w) => w,
        Err(error) => {
            handle_setup_error(app, error, true);
            return;
        }
    };
    window.present();

    // We have to await messages from the channel without blocking the main event loop
    glib::spawn_future_local(clone!( @weak window, @weak app => async move {
        while let Ok(message) = receiver.recv().await {
            match message {
                SelectMessage::Campaign { campaign } => {
                    start_dragon_display(&app, campaign).await;
                }
                SelectMessage::Remove => {
                    window.destroy();
                    remove_campaign(&app);
                },
                SelectMessage::Add => {
                    window.destroy();
                    add_campaign(&app);
                },
                SelectMessage::Error { error, fatal } => handle_setup_error(&app, error, fatal),
            }
        }
    }));
}

/// Make and present the window to remove a campaign
fn remove_campaign(app: &adw::Application) {
    let (sender, receiver) = async_channel::bounded(1);
    let window = match remove_campaign_window(app, sender) {
        Ok(w) => w,
        Err(error) => {
            handle_setup_error(app, error, true);
            return;
        }
    };
    window.present();

    // We have to await messages from the channel without blocking the main event loop
    glib::spawn_future_local(clone!( @weak window, @weak app => async move {
        while let Ok(message) = receiver.recv().await {
            match message {
                AddRemoveMessage::Campaign { campaign } => {
                    window.destroy();
                    match remove_campaign_from_config(campaign) {
                        Ok(_) => (),
                        Err(e) => handle_setup_error(&app, e, false),
                    }
                    remove_campaign(&app);
                }
                AddRemoveMessage::Cancel => {
                    window.destroy();
                    select_campaign(&app);
                },
                AddRemoveMessage::Error { error, fatal } => handle_setup_error(&app, error, fatal),
            }
        }
    }));
}

/// Make and present the window to add a campaign
fn add_campaign(app: &adw::Application) {
    let (sender, receiver) = async_channel::bounded(1);
    let window = match add_campaign_window(app, sender) {
        Ok(w) => w,
        Err(error) => {
            handle_setup_error(app, error, true);
            return;
        }
    };
    window.present();

    // We have to await messages from the channel without blocking the main event loop
    glib::spawn_future_local(clone!( @weak window, @weak app => async move {
        while let Ok(message) = receiver.recv().await {
            match message {
                AddRemoveMessage::Campaign { campaign } => {
                    window.destroy();
                    match campaign.sync_option {
                        SynchronizationOption::None => write_campaign(&app, campaign),
                        SynchronizationOption::GoogleDrive {..} => googledrive_connect(&app, campaign),
                    }
                }
                AddRemoveMessage::Cancel => {
                    window.destroy();
                    select_campaign(&app);
                },
                AddRemoveMessage::Error { error, fatal } => handle_setup_error(&app, error, fatal),
            }
        }
    }));
}

/// Call config processes to add a campaign to the config file
pub fn write_campaign(app: &adw::Application, campaign: Campaign) {
    match write_campaign_to_config(campaign) {
        Ok(_) => (),
        Err(e) => handle_setup_error(&app, e, false),
    }
    select_campaign(&app);
}

/// Make and present the window to connect to google drive
pub fn googledrive_connect(app: &adw::Application, campaign: Campaign) {
    let (sender, receiver) = async_channel::bounded(1);
    let window = match googledrive_connect_window(app, campaign, sender, false) {
        Ok(w) => w,
        Err(error) => {
            handle_setup_error(app, error, true);
            return;
        }
    };
    window.present();

    glib::spawn_future_local(clone!(@weak window, @weak app => async move {
        while let Ok(message) = receiver.recv().await {
            match message {
                AddRemoveMessage::Campaign { campaign } => {
                    window.destroy();
                    googledrive_select_path(&app, campaign);
                }
                AddRemoveMessage::Cancel => {
                    window.destroy();
                    select_campaign(&app);
                }
                AddRemoveMessage::Error { error, fatal } => handle_setup_error(&app, error, fatal),
            }
        }
    }));
}

/// Make and present the window to select a synchronization folder in google drive
pub fn googledrive_select_path(app: &adw::Application, campaign: Campaign) {
    let (sender, receiver) = async_channel::bounded(1);
    let window = match googledrive_select_path_window(app, campaign, sender) {
        Ok(w) => w,
        Err(error) => {
            handle_setup_error(app, error, true);
            return;
        }
    };
    window.present();

    glib::spawn_future_local(clone!(@weak window, @weak app => async move {
        while let Ok(message) = receiver.recv().await {
            match message {
                AddRemoveMessage::Campaign { campaign } => {
                    window.destroy();
                    write_campaign(&app, campaign);
                }
                AddRemoveMessage::Cancel => {
                    window.destroy();
                    select_campaign(&app);
                }
                AddRemoveMessage::Error { error, fatal } => handle_setup_error(&app, error, fatal),
            }
        }
    }));
}

pub async fn start_dragon_display(app: &adw::Application, campaign: Campaign) {
    // let monitor = select_monitor_window(&app);
    //checks if images need to be synchronized
    //starts the main application control panel
    //starts the main application image displayer
    todo!();
}

/// Function that produces proper error messages (dialogs) based on errors that are given
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
