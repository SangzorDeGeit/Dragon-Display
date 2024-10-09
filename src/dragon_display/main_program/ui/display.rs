use async_channel::Receiver;
use gtk::glib;
use gtk::glib::{clone, spawn_future_local};
use gtk::prelude::*;
use gtk::{ApplicationWindow, Picture};

use crate::dragon_display::main_program::Message;

pub fn display_window(app: &adw::Application, receiver: Receiver<Message>) -> ApplicationWindow {
    let window = ApplicationWindow::builder()
        .title("Dragon-Display")
        .application(app)
        .build();

    spawn_future_local(clone!(@weak window => async move  {
        while let Ok(message) = receiver.recv().await {
            match message {
                Message::Image { picture_path } => {
                    let picture = Picture::for_filename(picture_path);
                    window.set_child(Some(&picture));
                }
            }
        }
    }));

    window
}
