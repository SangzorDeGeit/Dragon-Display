use super::super::config::{read_campaign_from_config, MAX_CAMPAIGN_AMOUNT};
use super::super::SelectMessage;
use super::CustomMargin;
use crate::widgets::campaign_button::CampaignButton;

use gtk::glib::clone;
use gtk::prelude::*;
use gtk::{ApplicationWindow, Button, Grid, Label};

use async_channel::Sender;
use std::io::{Error, ErrorKind};

pub fn select_campaign_window(
    app: &adw::Application,
    sender: Sender<SelectMessage>,
) -> Result<ApplicationWindow, Error> {
    let container = Grid::new();
    container.set_vexpand(true);
    container.set_hexpand(true);
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Dragon-Display")
        .child(&container)
        .build();

    let mut max_campaigns_reached: bool = false;
    let label = Label::builder()
        .wrap(true)
        .max_width_chars(40)
        .hexpand_set(true)
        .vexpand_set(true)
        .build();
    label.set_margin_all(6);

    let button_add = Button::builder().label("add campaign").build();
    button_add.set_margin_all(6);

    let button_remove = Button::builder().label("remove campaign").build();
    button_remove.set_margin_all(6);

    let campaign_list = read_campaign_from_config()?;
    let mut i = 0;
    if campaign_list.len() == 0 {
        label.set_text("You have no campaigns yet");
    } else {
        label.set_text("Select a campaign");
        container.attach(&button_remove, i, 2, 1, 1);
    }
    // To add the campaign buttons
    for campaign in campaign_list {
        i += 1;
        let sender = sender.clone();
        let campaign_button = CampaignButton::new(campaign, Some(sender));
        container.attach(&campaign_button, i, 1, 1, 1)
    }

    // Center the label text based on the amount of campaigns
    if i % 2 == 0 {
        container.attach(&label, i / 2, 0, 2, 1);
    } else {
        container.attach(&label, (i / 2) + 1, 0, 1, 1);
    }

    container.attach(&button_add, i + 1, 2, 1, 1);

    if i >= i32::from(MAX_CAMPAIGN_AMOUNT) {
        max_campaigns_reached = true
    }

    container.set_halign(gtk::Align::Center);
    container.set_valign(gtk::Align::Center);

    button_add.connect_clicked(clone!(@strong sender => move |_| {
        if max_campaigns_reached{
            sender.send_blocking(SelectMessage::Error { error: Error::new(ErrorKind::OutOfMemory, "You cannot create anymore campaigns, maximum amount of campaigns reached!"), fatal: false}).expect("Channel was closed");
        }
        else {
            sender.send_blocking(SelectMessage::Add).expect("Channel was closed");
        }

    }));

    button_remove.connect_clicked(clone!(@strong sender => move |_| {
        sender.clone().send_blocking(SelectMessage::Remove).expect("Channel was closed");
    }));

    Ok(window)
}
