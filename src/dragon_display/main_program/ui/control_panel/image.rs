use std::fs;
use std::io::Error;

use async_channel::Sender;
use gtk::glib::clone;
use gtk::prelude::*;
use gtk::{Box, Grid, Image, Label, ToggleButton};

use crate::dragon_display::{main_program::Message, setup::config::Campaign};

pub const APP_ID: &str = "com.github.SangzorDeGeit.Dragon-Display";

pub fn create_image_page(campaign: Campaign, sender: Sender<Message>) -> Result<gtk::Box, Error> {
    let image_page = Box::new(gtk::Orientation::Horizontal, 6);
    let image_options = Box::new(gtk::Orientation::Vertical, 4);
    let imagegrid_navigation = Box::new(gtk::Orientation::Vertical, 6);
    image_page.append(&image_options);
    image_page.append(&imagegrid_navigation);
    let previous_next = Box::new(gtk::Orientation::Horizontal, 6);
    let image_grid = Grid::new();
    update_image_grid(campaign.clone(), sender.clone(), &image_grid)?;

    Ok(image_page)
}

/// Updates the imagegrid that is inputted
pub fn update_image_grid(
    campaign: Campaign,
    sender: Sender<Message>,
    grid: &Grid,
) -> Result<(), Error> {
    // first remove all children in the grid
    let mut current = match grid.first_child() {
        Some(c) => c,
        None => return create_image_grid(campaign, sender, grid),
    };
    let last_child = match grid.last_child() {
        Some(c) => c,
        None => return create_image_grid(campaign, sender, grid),
    };

    while !current.eq(&last_child) {
        let next = match current.next_sibling() {
            Some(c) => c,
            None => {
                return Err(Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "Could not find the last image in the grid",
                ))
            }
        };
        grid.remove(&current);
        current = next;
    }
    grid.remove(&current);
    return create_image_grid(campaign, sender, grid);
}

fn create_image_grid(
    campaign: Campaign,
    sender: Sender<Message>,
    grid: &Grid,
) -> Result<(), Error> {
    let settings = gtk::gio::Settings::new(APP_ID);
    let column = settings.int("imagegrid-column-amount");
    let row = settings.int("imagegrid-row-amount");
    assert!(
        column > 0 && row > 0,
        "image row or column is not greater then 0"
    );
    // create buttons for each image in the campaign.path
    let campaign_path = campaign.path;
    let files = fs::read_dir(campaign_path)?;
    let mut i = 0;
    let mut previous_button: Option<ToggleButton> = None;
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

        let button_child = Box::new(gtk::Orientation::Vertical, 2);
        button_child.set_vexpand(true);
        button_child.set_hexpand(true);
        let icon = Image::builder()
            .file(path.clone())
            .icon_size(gtk::IconSize::Inherit)
            .vexpand(true)
            .hexpand(true)
            .build();
        let file_name = file
            .file_name()
            .into_string()
            .expect("creating button label failed");
        let label = Label::builder().label(file_name).build();
        button_child.append(&icon);
        button_child.append(&label);
        let button = ToggleButton::builder().child(&button_child).build();
        grid.attach(&button, i % column, i / column, 1, 1);
        button.set_group(previous_button.as_ref());
        button.set_active(false);

        button.connect_clicked(clone!(@strong sender, @strong path => move |_| {
            sender
                .send_blocking(Message::Image { picture_path: path.to_string() })
                .expect("Channel closed");
        }));
        previous_button = Some(button);
        i += 1;
    }
    Ok(())
}
