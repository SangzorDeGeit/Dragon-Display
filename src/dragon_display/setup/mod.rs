pub mod gui;
pub mod config;
pub mod google_drive_sync;

use std::io::Error;

use adw::prelude::*;
use gtk::{glib, ApplicationWindow, Button, Grid, Label};
use config::{remove_campaign_from_config, write_campaign_to_config};
use serde::{Serialize, Deserialize};

use gui::{
    add_campaign_window, remove_campaign_window, select_campaign_window, select_monitor_window
};

/// Structure representing the name of the campaign and the corresponding data
#[derive(Serialize, Deserialize, Default)]
struct Config {
    campaigns: Vec<Campaign>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Campaign {
    pub name: String,
    pub path: String,
    pub sync_option: SynchronizationOption,
}

impl Campaign {
    /// Set the google drive properties of a google drive campaign. When this function is called on
    /// a non-google-drive campaign, it does nothing.
    pub fn set_google_drive_fields(&mut self, access_token: String, refresh_token: String, google_drive_sync_folder: String) -> &mut Self {
        let new_sync_option = SynchronizationOption::GoogleDrive { access_token, refresh_token, google_drive_sync_folder };
        match self.sync_option {
            SynchronizationOption::GoogleDrive { .. } => {
                self.sync_option = new_sync_option;
                return self;
            },
            _ => return self,
        }
    }

    /// set_google_drive_fields for just the tokens
    pub fn set_google_drive_tokens(&mut self, access_token: String, refresh_token: String) -> &mut Self {
        match &self.sync_option {
            SynchronizationOption::GoogleDrive { google_drive_sync_folder, .. } => {
                self.set_google_drive_fields(access_token, refresh_token, google_drive_sync_folder.to_string());
                return self;
            },
            _ => return self,
        }
    }

    /// set_google_drive_fields for just the sync_folder
    pub fn set_google_drive_sync_folder(&mut self, google_drive_sync_folder: String) -> &mut Self {
        match &self.sync_option {
            SynchronizationOption::GoogleDrive { access_token, refresh_token, .. } => {
                self.set_google_drive_fields(access_token.to_string(), refresh_token.to_string(), google_drive_sync_folder);
                return self;
            },
            _ => return self,
        }
    }
}

impl Default for Campaign {
    fn default() -> Self {
        Campaign { name: "".to_string(), path: "".to_string(), 
                sync_option: SynchronizationOption::None}
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SynchronizationOption {
    None,
    GoogleDrive {access_token: String,
                refresh_token: String,
                google_drive_sync_folder: String},
}

/// The messages that the select_campaign_window can send
pub enum SelectMessage {
    Campaign { campaign: Campaign },
    Remove,
    Add,
    Error { error: Error, fatal: bool },
}

/// The messages that the add campaign window can send
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
            handle_setup_error(app, error, true);
            return;
        }
    };
    window.present();

    // We have to await messages from the channel without blocking the main event loop
    glib::spawn_future_local(glib::clone!( @weak window, @weak app => async move {
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
    glib::spawn_future_local(glib::clone!( @weak window, @weak app => async move {
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
    glib::spawn_future_local(glib::clone!( @weak window, @weak app => async move {
        while let Ok(message) = receiver.recv().await {
            match message {
                AddRemoveMessage::Campaign { campaign } => {
                    window.destroy();
                    match write_campaign_to_config(campaign) {
                        Ok(_) => (),
                        Err(e) => handle_setup_error(&app, e, false),
                    }
                    select_campaign(&app);
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

pub async fn start_dragon_display(app: &adw::Application, campaign: Campaign) {
    // let monitor = select_monitor_window(&app);
    //checks if images need to be synchronized
    //starts the main application control panel
    //starts the main application image displayer
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
