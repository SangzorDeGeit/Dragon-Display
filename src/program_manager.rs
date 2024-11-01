use gtk::glib::clone;
use image::metadata::Orientation;
use std::{
    cell::RefCell,
    io::{Error, ErrorKind},
    rc::Rc,
};

use crate::{
    config::{write_campaign_to_config, Campaign},
    setup_manager::{googledrive_connect, CallingFunction},
    ui::{
        control_window::ControlWindow, display_window::DdDisplayWindow, error_dialog::ErrorDialog,
        googledrive_synchronize::GoogledriveSynchronizeWindow, options::DdOptionsWindow,
    },
};

use async_channel::Sender;
use gdk4::Monitor;
use gtk::{glib::spawn_future_local, prelude::*};

pub enum DisplayWindowMessage {
    Image { picture_path: String },
    Fit { fit: bool },
    Video { video_path: String },
}

pub enum ControlWindowMessage {
    Image { picture_path: String },
    Fit { fit: bool },
    Video { video_path: String },
    Refresh { sender: Sender<()> },
    Options { sender: Sender<()> },
    Rotate { orientation: Orientation },
    Error { error: Error, fatal: bool },
}

pub fn dragon_display(app: &adw::Application, campaign: Campaign, monitor: Monitor) {
    let (control_sender, control_receiver) = async_channel::unbounded();
    let (display_sender, display_receiver) = async_channel::unbounded();

    let control_window = ControlWindow::new(&app, campaign.clone(), control_sender);
    control_window.present();
    control_window.maximize();
    let display_window = DdDisplayWindow::new(&app, display_receiver);
    display_window.present();
    display_window.fullscreen_on_monitor(&monitor);
    let current_path = Rc::from(RefCell::from("".to_string()));

    spawn_future_local(clone!(@strong app, @strong current_path => async move {
        while let Ok(message) = control_receiver.recv().await {
            match message {
                ControlWindowMessage::Image { picture_path } => {
                    current_path.replace(picture_path.clone());
                    display_sender
                    .send_blocking(DisplayWindowMessage::Image { picture_path })
                    .expect("Channel closed");
                },
                ControlWindowMessage::Fit { fit } => display_sender.send_blocking(DisplayWindowMessage::Fit { fit }).expect("Channel closed"),
                ControlWindowMessage::Video { video_path } => {
                    current_path.replace(video_path.clone());
                    display_sender.send_blocking(DisplayWindowMessage::Video { video_path })
                    .expect("Channel closed");
                },
                ControlWindowMessage::Refresh { sender } => refresh(&app, campaign.clone(), sender),
                ControlWindowMessage::Options { sender } => options(&app, sender),
                ControlWindowMessage::Rotate { orientation }=> {
                    match rotate(&current_path.borrow(), orientation) {
                        Ok(_) => display_sender.send_blocking(DisplayWindowMessage::Image { picture_path: current_path.borrow().to_string() }).expect("Channel closed"),
                        Err(error) => ErrorDialog::new(&app, error, false).present(),
                    };
                }
                ControlWindowMessage::Error { error, fatal } => ErrorDialog::new(&app, error, fatal).present(),

            }
        }
    }));
    control_window.connect_destroy(clone!(@strong app, @strong control_window => move |_| {
        display_window.destroy();
    }));
}

pub fn refresh(app: &adw::Application, campaign: Campaign, sender: Sender<()>) {
    let (sync_sender, sync_receiver) = async_channel::unbounded();
    // call make a google_synchronize window
    let window = GoogledriveSynchronizeWindow::new(&app, campaign.clone(), sync_sender);
    window.present();
    spawn_future_local(clone!(@strong sender, @strong app => async move {
        while let Ok(message) = sync_receiver.recv().await {
            match message {
                Ok((campaign, failed)) => {
                    window.destroy();
                    if failed.len() > 0 {
                        let failed_files = failed.join(", ");
                        let errormsg = format!(
                            "The following files could not be downloaded:\n{}",
                            failed_files
                        );
                        ErrorDialog::new(&app, Error::new(ErrorKind::Unsupported, errormsg), false)
                            .present();
                    }
                    match write_campaign_to_config(campaign.clone()) {
                        Ok(_) => sender.send(()).await.expect("Channel closed"),
                        Err(error) => ErrorDialog::new(&app, error, true).present(),
                    }
                }
                Err(e) => match e.kind() {
                    ErrorKind::ConnectionRefused => {
                        window.destroy();
                        googledrive_connect(
                            &app,
                            campaign.clone(),
                            CallingFunction::Refresh { sender: sender.clone() },
                        )
                    }
                    _ => ErrorDialog::new(&app, e, false).present(),
                },
            }
        }
    }));
}

fn options(app: &adw::Application, sender: Sender<()>) {
    let window = DdOptionsWindow::new(&app, sender);
    window.present();
}

/// rotates the file at path and overwrites it
fn rotate(path: &str, orientation: Orientation) -> Result<(), Error> {
    let mut image = match image::open(path) {
        Ok(image) => image,
        Err(e) => return Err(Error::new(ErrorKind::InvalidData, e.to_string())),
    };
    image.apply_orientation(orientation);
    match image.save(path) {
        Ok(_) => (),
        Err(e) => return Err(Error::new(ErrorKind::WriteZero, e.to_string())),
    }
    Ok(())
}
