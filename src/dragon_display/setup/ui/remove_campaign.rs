use crate::dragon_display::setup::config::{read_campaign_from_config, Campaign};
use crate::dragon_display::setup::ui::CustomMargin;
use crate::dragon_display::setup::AddRemoveMessage;
use crate::widgets::remove_button::RemoveButton;

use gtk::glib::{clone, spawn_future_local};
use gtk::prelude::*;
use gtk::{ApplicationWindow, Button, Grid, Label};

use async_channel::Sender;
use std::io::Error;
// The 'remove campaign' window
pub fn remove_campaign_window(
    app: &adw::Application,
    sender: Sender<AddRemoveMessage>,
) -> Result<ApplicationWindow, Error> {
    let campaign_list = read_campaign_from_config()?;
    if campaign_list.len() == 0 {
        sender
            .send_blocking(AddRemoveMessage::Cancel)
            .expect("Channel closed");
    }

    let label = Label::builder()
        .label("Select the campaign you want to remove")
        .build();
    label.set_margin_all(6);

    let button_cancel = Button::builder().label("Cancel").build();
    button_cancel.set_margin_all(6);

    let container = Grid::new();
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Dragon-Display")
        .resizable(false)
        .child(&container)
        .build();

    let (button_sender, button_receiver) = async_channel::bounded(1);

    let mut i = 0;
    for campaign in campaign_list {
        let campaign_button = RemoveButton::new(campaign, Some(button_sender.clone()));
        campaign_button.set_halign(gtk::Align::Center);
        container.attach(&campaign_button, i, 1, 1, 1);
        i += 1;
    }
    i = -1;

    if i % 2 == 0 {
        container.attach(&label, i / 2, 0, 1, 1);
        container.attach(&button_cancel, i / 2, 2, 1, 1);
    } else {
        container.attach(&label, i / 2, 0, 2, 1);
        container.attach(&button_cancel, i / 2, 2, 2, 1);
    }

    button_cancel.connect_clicked(clone!(@strong sender => move |_| {
        sender.send_blocking(AddRemoveMessage::Cancel).expect("Channel closed");
    }));

    spawn_future_local(clone!(@strong app, @strong sender => async move {
        while let Ok(message) = button_receiver.recv().await {
            remove_campaign_confirm(&app, message, sender.clone())
        }
    }));

    Ok(window)
}

fn remove_campaign_confirm(
    app: &adw::Application,
    campaign: Campaign,
    sender: Sender<AddRemoveMessage>,
) {
    let message = format!(
        "Are you sure you want to delete {}?",
        campaign.name.as_str()
    );
    let label = Label::builder().label(message).build();
    label.set_margin_all(6);

    let button_yes = Button::builder().label("Yes").build();
    button_yes.set_margin_all(6);

    let button_no = Button::builder().label("No").build();
    button_no.set_margin_all(6);

    let container = Grid::new();
    let confirm_window = ApplicationWindow::builder()
        .application(app)
        .modal(true)
        .title("Dragon-Display")
        .resizable(false)
        .child(&container)
        .build();

    container.attach(&label, 0, 0, 2, 1);
    container.attach(&button_yes, 1, 1, 1, 1);
    container.attach(&button_no, 0, 1, 1, 1);

    button_yes.connect_clicked(clone!(@strong confirm_window, @strong sender => move |_| {
        sender.send_blocking(AddRemoveMessage::Campaign { campaign: campaign.clone() }).expect("Channel Closed");
        confirm_window.close();
    }));

    button_no.connect_clicked(clone!(@strong confirm_window => move |_| {
        confirm_window.close();
    }));

    confirm_window.present();
}
