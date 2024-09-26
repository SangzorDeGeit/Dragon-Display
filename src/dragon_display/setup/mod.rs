pub mod config;
pub mod google_drive;
pub mod ui;

use std::fs;
use std::io::{Error, ErrorKind};

use adw::prelude::*;
use config::{remove_campaign_from_config, write_campaign_to_config};
use glib::clone;
use gtk::glib;
use gtk::glib::spawn_future_local;

use config::{Campaign, SynchronizationOption};

use ui::add_campaign::add_campaign_window;
use ui::error::handle_setup_error;
use ui::google_drive::{
    googledrive_connect_window, googledrive_select_path_window, googledrive_synchronize_window,
};
use ui::remove_campaign::remove_campaign_window;
use ui::select_campaign::select_campaign_window;
use ui::select_monitor::select_monitor_window;

use crate::dragon_display::main_program::start_dragon_display;

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

pub enum CallingFunction {
    AddCampaign,
    SelectPath,
    Synchronize,
    //Refresh,
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
                    window.destroy();
                    match campaign.sync_option {
                        SynchronizationOption::None => select_monitor(&app, campaign),
                        SynchronizationOption::GoogleDrive { .. } => googledrive_synchronize(&app, campaign),
                    }
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
                        SynchronizationOption::GoogleDrive {..} => googledrive_connect(&app, campaign, CallingFunction::AddCampaign),
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
    if let Err(e) = write_campaign_to_config(campaign) {
        handle_setup_error(&app, e, false);
    }
    select_campaign(&app);
}

/// Make and present the window to connect to google drive
pub fn googledrive_connect(
    app: &adw::Application,
    campaign: Campaign,
    calling_function: CallingFunction,
) {
    let (sender, receiver) = async_channel::bounded(1);
    let reconnect = match calling_function {
        CallingFunction::SelectPath => true,
        CallingFunction::Synchronize => true,
        CallingFunction::AddCampaign => false,
    };
    let window = match googledrive_connect_window(app, campaign.clone(), sender, reconnect) {
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
                    match calling_function {
                        CallingFunction::AddCampaign => googledrive_select_path(&app, campaign),
                        CallingFunction::SelectPath => googledrive_select_path(&app, campaign),
                        CallingFunction::Synchronize => googledrive_synchronize(&app, campaign),
                    }
                }
                AddRemoveMessage::Cancel => {
                    window.destroy();
                }
                AddRemoveMessage::Error { error, fatal } => handle_setup_error(&app, error, fatal),
            }
        }
    }));
}

/// Make and present the window to select a synchronization folder in google drive
pub fn googledrive_select_path(app: &adw::Application, campaign: Campaign) {
    let (sender, receiver) = async_channel::bounded(1);
    let window = match googledrive_select_path_window(app, campaign.clone(), sender) {
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
                AddRemoveMessage::Error { error, fatal } => {
                    match error.kind() {
                        ErrorKind::ConnectionRefused => {
                            window.destroy();
                            googledrive_connect(&app, campaign.clone(), CallingFunction::SelectPath);
                        },
                        _ => handle_setup_error(&app, error, fatal),
                    }
                }
            }
        }
    }));
}

fn googledrive_synchronize(app: &adw::Application, campaign: Campaign) {
    let (sender, receiver) = async_channel::bounded::<Result<(Campaign, Vec<String>), Error>>(1);
    let window = googledrive_synchronize_window(&app, campaign.clone(), sender);
    window.present();

    if let Err(e) = fs::create_dir_all(&campaign.path) {
        handle_setup_error(
            app,
            Error::new(e.kind(), "Could not create folder for images"),
            true,
        );
    }

    glib::spawn_future_local(clone!(@strong campaign, @weak app => async move {
        while let Ok(m) = receiver.recv().await {
            match m {
                Ok((campaign, failed)) => {
                    window.destroy();
                    if failed.len() > 0 {
                        let failed_files = failed.join(", ");
                        let errormsg = format!("The following files could not be downloaded:\n{}", failed_files);
                        handle_setup_error(&app, Error::new(ErrorKind::Unsupported, errormsg), false);
                    }
                    select_monitor(&app, campaign);
                },
                Err(e) => match e.kind() {
                    ErrorKind::ConnectionRefused => {
                        window.destroy();
                        googledrive_connect(&app, campaign.clone(), CallingFunction::Synchronize)
                    }
                    _ => handle_setup_error(&app, e, false),
                },
            };
        }
    }));
}

fn select_monitor(app: &adw::Application, campaign: Campaign) {
    let (sender, receiver) = async_channel::bounded(1);
    let window = match select_monitor_window(app, sender) {
        Ok(w) => w,
        Err(e) => {
            handle_setup_error(&app, e, true);
            return;
        }
    };
    window.present();

    spawn_future_local(clone!(@weak window, @weak app => async move {
        while let Ok(monitor) = receiver.recv().await {
            window.destroy();
            start_dragon_display(&app, campaign.clone(), monitor);
        };
    }));
}
