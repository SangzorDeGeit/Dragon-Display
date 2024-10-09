pub mod image;
pub mod uvtt;
pub mod video;

use gtk::glib::{self, clone, spawn_future_local};
use gtk::{prelude::*, Stack};
use gtk::{ApplicationWindow, Box, Button, Grid};

use async_channel::Sender;
use std::io::{Error, ErrorKind};

use crate::dragon_display::main_program::Message;
use crate::dragon_display::setup::config::{
    write_campaign_to_config, Campaign, SynchronizationOption,
};
use crate::dragon_display::setup::ui::google_drive::googledrive_synchronize_window;
use crate::dragon_display::setup::{googledrive_connect, CallingFunction};
use image::{create_image_page, update_image_grid};

use crate::ui::error_dialog::ErrorDialog;

pub fn control_panel_window(
    app: &adw::Application,
    campaign: Campaign,
    sender: Sender<Message>,
) -> Result<ApplicationWindow, Error> {
    let container = Grid::new();
    let window = ApplicationWindow::builder()
        .title("Dragon-display")
        .application(app)
        .child(&container)
        .maximized(true)
        .build();
    // create the stack that contains the image, video and uvtt page
    let stack = Stack::new();

    let image_page = create_image_page(campaign.clone(), sender.clone());

    // create the basic buttons
    let global_functions = Box::new(gtk::Orientation::Horizontal, 3);
    global_functions.set_hexpand(true);
    container.attach(&global_functions, 0, 0, 1, 1);
    let refresh_button = Button::builder().label("Refresh").hexpand(true).build();
    let options_button = Button::builder().label("Rotate").hexpand(true).build();
    global_functions.append(&refresh_button);
    global_functions.append(&options_button);

    let image_grid = Grid::new();
    update_image_grid(campaign.clone(), sender.clone(), &image_grid)?;

    let (refresh_sender, refresh_receiver) = async_channel::unbounded();
    refresh_button.connect_clicked(
        clone!(@weak app, @strong campaign, @strong sender, @weak image_grid => move |_| {
            if let SynchronizationOption::GoogleDrive { .. } = campaign.sync_option {
                refresh(&app, campaign.clone(), refresh_sender.clone());
            } else {
                let _ = update_image_grid(campaign.clone(), sender.clone(), &image_grid);
            }
        }),
    );

    container.attach(&image_grid, 0, 1, 1, 1);

    spawn_future_local(async move {
        while let Ok(_) = refresh_receiver.recv().await {
            match update_image_grid(campaign.clone(), sender.clone(), &image_grid) {
                Ok(_) => (),
                Err(_) => todo!("could not update grid: do some error handling maybe"),
            }
        }
    });

    Ok(window)
}

pub fn refresh(app: &adw::Application, campaign: Campaign, refresh_sender: Sender<()>) {
    let (sender, receiver) = async_channel::bounded(1);
    let window = googledrive_synchronize_window(app, campaign.clone(), sender);
    window.present();

    spawn_future_local(clone!(@strong campaign, @weak app => async move {
        while let Ok(message) = receiver.recv().await {
            match message {
                Ok((campaign, failed)) => {
                    window.destroy();
                    if failed.len() > 0 {
                        let failed_files = failed.join(", ");
                        let errormsg = format!("The following files could not be downloaded:\n{}", failed_files);
                        ErrorDialog::new(&app, Error::new(ErrorKind::Unsupported, errormsg), false);
                    }
                    match write_campaign_to_config(campaign) {
                        Ok(_) => (),
                        Err(e) => ErrorDialog::new(&app, e, true).present(),
                    }
                    refresh_sender.send_blocking(()).expect("Channel closed");
                },
                Err(e) => match e.kind() {
                    ErrorKind::ConnectionRefused => {
                        window.destroy();
                        googledrive_connect(&app, campaign.clone(), CallingFunction::Refresh, Some(refresh_sender.clone()));
                    }
                    _ => ErrorDialog::new(&app, e, false).present()
                },
            }
        }
    }));
}
