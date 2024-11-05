use gtk::glib::clone;
use gtk::{gdk_pixbuf::PixbufRotation, glib::timeout_future_seconds};
use std::cell::Cell;
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
use gdk4::{Display, Monitor};
use gtk::{glib::spawn_future_local, prelude::*};

pub enum DisplayWindowMessage {
    Image { picture_path: String },
    Fit { fit: bool },
    Rotate { rotation: PixbufRotation },
    Video { video_path: String },
}

pub enum ControlWindowMessage {
    Image { picture_path: String },
    Fit { fit: bool },
    Video { video_path: String },
    Refresh { sender: Sender<()> },
    Options { sender: Sender<()> },
    Rotate { rotation: PixbufRotation },
    Error { error: Error, fatal: bool },
}

/// Makes the control and display window and sends signals between them
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
    let display = monitor.display();
    let current_rotation = Rc::from(Cell::from(0));

    // Connect control window signals to display window update
    spawn_future_local(
        clone!(@strong app, @strong current_path, @strong current_rotation => async move {
            while let Ok(message) = control_receiver.recv().await {
                match message {
                    ControlWindowMessage::Image { picture_path } => {
                        current_path.replace(picture_path.clone());
                        current_rotation.set(0);
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
                    ControlWindowMessage::Rotate { rotation }=> {
                        let rotation = get_rotation(current_rotation.clone(), rotation);
                        display_sender.send_blocking(DisplayWindowMessage::Rotate { rotation }).expect("Channel closed");
                    },
                    ControlWindowMessage::Error { error, fatal } => ErrorDialog::new(&app, error, fatal).present(),

                }
            }
        }),
    );

    control_window.connect_destroy(
        clone!(@strong app, @strong control_window, @strong display_window => move |_| {
            display_window.destroy();
        }),
    );

    update_display(display, display_window, monitor);
}

/// Handles refresh request
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

/// Handles option request
fn options(app: &adw::Application, sender: Sender<()>) {
    let window = DdOptionsWindow::new(&app, sender);
    window.present();
}

/// Takes a rotate operations from control and applies it to the current rotation
fn get_rotation(current_rotation: Rc<Cell<u32>>, rotation: PixbufRotation) -> PixbufRotation {
    match rotation {
        PixbufRotation::None => current_rotation.set((current_rotation.get() + 0) % 360),
        PixbufRotation::Clockwise => current_rotation.set((current_rotation.get() + 90) % 360),
        PixbufRotation::Upsidedown => current_rotation.set((current_rotation.get() + 180) % 360),
        PixbufRotation::Counterclockwise => {
            current_rotation.set((current_rotation.get() + 270) % 360)
        }
        _ => panic!("invalid rotation given"),
    }
    match current_rotation.get() {
        0 => PixbufRotation::None,
        90 => PixbufRotation::Clockwise,
        180 => PixbufRotation::Upsidedown,
        270 => PixbufRotation::Counterclockwise,
        _ => panic!("resulted into an invalid rotation"),
    }
}

/// Checks every 5 seconds if the display monitor has disconnected and unfullscreens if it does so
/// Does fullscreen on newly connected monitor
fn update_display(display: Display, display_window: DdDisplayWindow, monitor: Monitor) {
    spawn_future_local(async move {
        let mut connected_monitors: Vec<Monitor>;
        let mut monitor = monitor;
        loop {
            connected_monitors = display
                .monitors()
                .into_iter()
                .filter_map(|m| m.ok())
                .filter_map(|m| m.to_value().get::<Monitor>().ok())
                .collect();
            timeout_future_seconds(4).await;
            if (connected_monitors.len() as u32) > display.monitors().n_items()
                && !monitor.is_valid()
            {
                display_window.unfullscreen();
            }
            if (connected_monitors.len() as u32) < display.monitors().n_items() {
                monitor = display
                    .monitors()
                    .into_iter()
                    .filter_map(|m| m.ok())
                    .filter_map(|m| m.to_value().get::<Monitor>().ok())
                    .find(|m| !connected_monitors.contains(m))
                    .expect("Could not get new_monitor");
                display_window.fullscreen_on_monitor(&monitor);
            }
        }
    });
}
