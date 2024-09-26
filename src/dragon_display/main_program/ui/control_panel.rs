use gtk::glib::clone;
use gtk::prelude::*;
use gtk::{ApplicationWindow, Box, Button};

use async_channel::Sender;
use std::{fs, io::Error};

use crate::dragon_display::main_program::Message;
use crate::dragon_display::setup::config::Campaign;

pub fn control_panel_window(
    app: &adw::Application,
    campaign: Campaign,
    sender: Sender<Message>,
) -> Result<ApplicationWindow, Error> {
    let container = Box::new(gtk::Orientation::Vertical, 1);
    let window = ApplicationWindow::builder()
        .title("Dragon-display")
        .application(app)
        .child(&container)
        .maximized(true)
        .build();

    // create buttons for each image in the campaign.path
    let files = fs::read_dir(campaign.path)?;
    for file in files {
        let file = match file {
            Ok(f) => f,
            Err(_) => continue,
        };
        let file_path = file.path();
        let path = file_path
            .to_str()
            .expect("failed to convert path to string")
            .to_string();
        let button = Button::builder()
            .label(
                file.file_name()
                    .into_string()
                    .expect("creating button label failed"),
            )
            .build();
        container.append(&button);

        button.connect_clicked(clone!(@strong sender, @strong path => move |_| {
            sender
                .send_blocking(Message::Image { picture_path: path.to_string() })
                .expect("Channel closed");
        }));
    }

    Ok(window)
}
