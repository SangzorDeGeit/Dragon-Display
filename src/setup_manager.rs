use std::{
    fs,
    io::{Error, ErrorKind},
};

use super::config::{remove_campaign_from_config, write_campaign_to_config};
use adw::prelude::*;
use async_channel::Sender;
use glib::clone;
use gtk::glib;
use gtk::glib::spawn_future_local;

use crate::config;
use crate::config::{Campaign, SynchronizationOption};
use crate::program_manager::dragon_display;

use crate::ui::add_campaign::AddCampaignWindow;
use crate::ui::error_dialog::ErrorDialog;
use crate::ui::googledrive_connect::GoogledriveConnectWindow;
use crate::ui::googledrive_select_folder::DdGoogleFolderSelectWindow;
use crate::ui::googledrive_synchronize::GoogledriveSynchronizeWindow;
use crate::ui::remove_campaign::RemoveCampaignWindow;
use crate::ui::remove_confirm::RemoveConfirmWindow;
use crate::ui::select_campaign::SelectCampaignWindow;
use crate::ui::select_monitor::SelectMonitorWindow;

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
}

/// Make and present the window to select a campaign
pub fn select_campaign(app: &adw::Application) {
    // This channel lets the frontend(ui) communicate with the manager
    let (sender, receiver) = async_channel::bounded(1);
    let campaign_list = match config::read_campaign_from_config() {
        Ok(l) => l,
        Err(error) => {
            ErrorDialog::new(app, error, true).present();
            return;
        }
    };
    let window = SelectCampaignWindow::new(app, Some(sender), campaign_list);
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
                SelectMessage::Error { error, fatal } => ErrorDialog::new(&app, error, fatal).present(),
            }
        }
    }));
}

/// Make and present the window to remove a campaign
fn remove_campaign(app: &adw::Application) {
    // This channel lets the frontend(ui) communicate with the manager
    let (sender, receiver) = async_channel::bounded(1);
    let campaign_list = match config::read_campaign_from_config() {
        Ok(l) => l,
        Err(error) => {
            ErrorDialog::new(app, error, true).present();
            return;
        }
    };
    let window = RemoveCampaignWindow::new(app, Some(sender), campaign_list);
    window.present();

    // We have to await messages from the channel without blocking the main event loop
    glib::spawn_future_local(clone!( @weak window, @weak app => async move {
        while let Ok(message) = receiver.recv().await {
            match message {
                AddRemoveMessage::Campaign { campaign } => {
                    window.destroy();
                    remove_confirm(&app, campaign);
                }
                AddRemoveMessage::Cancel => {
                    window.destroy();
                    select_campaign(&app);
                },
                AddRemoveMessage::Error { error, fatal } => ErrorDialog::new(&app, error, fatal).present(),
            }
        }
    }));
}

fn remove_confirm(app: &adw::Application, campaign: Campaign) {
    // This channel lets the frontend(ui) communicate with the manager
    let (sender, receiver) = async_channel::bounded(1);
    let window = RemoveConfirmWindow::new(app, Some(sender), campaign);
    window.present();

    glib::spawn_future_local(clone!( @weak window, @weak app => async move {
        while let Ok(message) = receiver.recv().await {
            match message {
                AddRemoveMessage::Campaign { campaign } => {
                    window.destroy();
                    match remove_campaign_from_config(campaign, true) {
                        Ok(_) => (),
                        Err(error) => ErrorDialog::new(&app, error, false).present(),
                    }
                    remove_campaign(&app);
                }
                AddRemoveMessage::Cancel => {
                    window.destroy();
                    remove_campaign(&app);
                },
                AddRemoveMessage::Error { error, fatal } => ErrorDialog::new(&app, error, fatal).present(),
            }
        }
    }));
}

/// Make and present the window to add a campaign
fn add_campaign(app: &adw::Application) {
    let (sender, receiver) = async_channel::unbounded();
    let window = AddCampaignWindow::new(&app, Some(sender));
    window.present();

    // We have to await messages from the channel without blocking the main event loop
    glib::spawn_future_local(clone!( @weak window, @weak app => async move {
        while let Ok(message) = receiver.recv().await {
            match message {
                AddRemoveMessage::Campaign { campaign } => {
                    window.destroy();
                    match campaign.sync_option {
                        SynchronizationOption::None => write_campaign(&app, campaign),
                        SynchronizationOption::GoogleDrive {..} => googledrive_connect(&app, campaign, CallingFunction::AddCampaign, None),
                    }
                }
                AddRemoveMessage::Cancel => {
                    window.destroy();
                    select_campaign(&app);
                },
                AddRemoveMessage::Error { error, fatal } => ErrorDialog::new(&app, error, fatal).present(),
            }
        }
    }));
}

/// Call config processes to add a campaign to the config file
pub fn write_campaign(app: &adw::Application, campaign: Campaign) {
    if let Err(error) = write_campaign_to_config(campaign) {
        ErrorDialog::new(app, error, true).present();
    }
    select_campaign(&app);
}

/// Make and present the window to connect to google drive
pub fn googledrive_connect(
    app: &adw::Application,
    campaign: Campaign,
    calling_function: CallingFunction,
    refresh_sender: Option<Sender<()>>,
) {
    let (sender, receiver) = async_channel::bounded(1);
    let reconnect = match calling_function {
        CallingFunction::SelectPath => true,
        CallingFunction::Synchronize => true,
        CallingFunction::AddCampaign => false,
    };
    let window = GoogledriveConnectWindow::new(&app, campaign.clone(), sender, reconnect);
    window.present();

    glib::spawn_future_local(
        clone!(@weak window, @weak app, @strong campaign => async move {
            while let Ok(message) = receiver.recv().await {
                match message {
                    AddRemoveMessage::Campaign { campaign } => {
                        window.destroy();
                        match calling_function {
                            CallingFunction::AddCampaign => googledrive_select_folder(&app, campaign),
                            CallingFunction::SelectPath => googledrive_select_folder(&app, campaign),
                            CallingFunction::Synchronize => googledrive_synchronize(&app, campaign),
                        }
                    }
                    AddRemoveMessage::Cancel => {
                        window.destroy();
                        match calling_function {
                            CallingFunction::AddCampaign => select_campaign(&app),
                            CallingFunction::SelectPath => googledrive_select_folder(&app, campaign.clone()),
                            CallingFunction::Synchronize => googledrive_synchronize(&app, campaign.clone()),
                        }
                    }
                    AddRemoveMessage::Error { error, fatal } => ErrorDialog::new(&app, error, fatal).present(),
                }
            }
        }),
    );
}

/// Make and present the window to select a synchronization folder in google drive
pub fn googledrive_select_folder(app: &adw::Application, campaign: Campaign) {
    let (sender, receiver) = async_channel::bounded(1);
    let window = DdGoogleFolderSelectWindow::new(app, campaign.clone(), sender);
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
                            googledrive_connect(&app, campaign.clone(), CallingFunction::SelectPath, None);
                        },
                        _ => ErrorDialog::new(&app, error, fatal).present(),
                    }
                }
            }
        }
    }));
}

fn googledrive_synchronize(app: &adw::Application, campaign: Campaign) {
    let (sender, receiver) = async_channel::bounded::<Result<(Campaign, Vec<String>), Error>>(1);
    let window = GoogledriveSynchronizeWindow::new(app, campaign.clone(), sender);
    window.present();

    if let Err(e) = fs::create_dir_all(&campaign.path) {
        ErrorDialog::new(
            app,
            Error::new(e.kind(), "Could not create folder for images"),
            true,
        )
        .present();
    }

    glib::spawn_future_local(clone!(@strong campaign, @weak app => async move {
        while let Ok(m) = receiver.recv().await {
            match m {
                Ok((campaign, failed)) => {
                    window.destroy();
                    if failed.len() > 0 {
                        let failed_files = failed.join(", ");
                        let errormsg = format!("The following files could not be downloaded:\n{}", failed_files);
                        ErrorDialog::new(&app, Error::new(ErrorKind::Unsupported, errormsg), false).present();
                    }
                    match write_campaign_to_config(campaign.clone()) {
                        Ok(_) => select_monitor(&app, campaign),
                        Err(error) => ErrorDialog::new(&app, error, true).present(),
                    }
                },
                Err(e) => match e.kind() {
                    ErrorKind::ConnectionRefused => {
                        window.destroy();
                        googledrive_connect(&app, campaign.clone(), CallingFunction::Synchronize, None)
                    }
                    _ => ErrorDialog::new(&app, e, false).present(),
                },
            };
        }
    }));
}

fn select_monitor(app: &adw::Application, campaign: Campaign) {
    let (sender, receiver) = async_channel::bounded(1);
    let window = SelectMonitorWindow::new(app, sender);
    window.present();

    spawn_future_local(clone!(@weak window, @weak app => async move {
        while let Ok(message) = receiver.recv().await {
            match message {
                Ok(monitor) => {
                    window.destroy();
                    dragon_display(&app, campaign.clone(), monitor)
                },
                Err(error) => {
                    ErrorDialog::new(&app, error, true).present();
                    return;
                }
            }
        }
    }));
}
