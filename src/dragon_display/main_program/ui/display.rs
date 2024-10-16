use async_channel::Receiver;
use gtk::glib;
use gtk::glib::{clone, spawn_future_local};
use gtk::prelude::*;
use gtk::{ApplicationWindow, Picture};

use crate::ui::control_window::UpdateDisplayMessage;

pub fn display_window(
    app: &adw::Application,
    receiver: Receiver<UpdateDisplayMessage>,
) -> ApplicationWindow {
    let window = ApplicationWindow::builder()
        .title("Dragon-Display")
        .application(app)
        .build();

    spawn_future_local(clone!(@weak window => async move  {
        while let Ok(message) = receiver.recv().await {
            match message {
                UpdateDisplayMessage::Image { picture_path } => {
                    let picture = Picture::for_filename(picture_path);
                    window.set_child(Some(&picture));
                }
                UpdateDisplayMessage::Error { error, fatal } => todo!("react on errors"),
            }
        }
    }));

    window
}
