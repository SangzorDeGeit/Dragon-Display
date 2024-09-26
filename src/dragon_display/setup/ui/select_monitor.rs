use std::io::{Error, ErrorKind};

// packages for gui
use adw::prelude::*;
use async_channel::Sender;
use gdk4::{Display, Monitor};
use glib::clone;
use gtk::{glib, ApplicationWindow, Button, Grid, Label};

pub fn select_monitor_window(
    app: &adw::Application,
    sender: Sender<Monitor>,
) -> Result<ApplicationWindow, Error> {
    let container = Grid::new();
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Dragon-Display")
        .child(&container)
        .build();

    let label = Label::builder()
        .label("Choose the screen that you want to display the images on")
        .margin_start(6)
        .margin_end(6)
        .margin_top(6)
        .margin_bottom(6)
        .build();

    let display = match Display::default() {
        Some(d) => d,
        None => {
            return Err(Error::new(
                ErrorKind::NotFound,
                "Could not find any displays",
            ))
        }
    };

    let mut i: u32 = 0;
    while let Some(monitor) = display.monitors().item(i) {
        let monitor = monitor
            .to_value()
            .get::<Monitor>()
            .expect("Value needs to be monitor");

        let monitor_button = Button::builder()
            .label(
                format!(
                    "{}cm x {}cm",
                    monitor.height_mm() / 10,
                    monitor.width_mm() / 10
                )
                .as_str(),
            )
            .margin_start(6)
            .margin_end(6)
            .margin_top(6)
            .margin_bottom(6)
            .build();

        monitor_button.connect_clicked(clone!(@strong sender, @weak monitor => move |_| {
            sender.send_blocking(monitor).expect("Channel closed");
        }));

        let column = match i32::try_from(i) {
            Ok(c) => c,
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::OutOfMemory,
                    "Found to many monitors?",
                ))
            }
        };
        container.attach(&monitor_button, column, 1, 1, 1);

        i = i + 1;
    }
    let monitor_amount = match i32::try_from(i) {
        Ok(c) => c,
        Err(_) => {
            return Err(Error::new(
                ErrorKind::OutOfMemory,
                "Too many monitors found",
            ));
        }
    };
    if monitor_amount == 0 {
        label.set_text("Could not detect any monitors");
        container.attach(&label, 0, 0, 0, 0);
    } else if monitor_amount % 2 == 1 {
        container.attach(&label, monitor_amount / 2, 0, 1, 1);
    } else {
        container.attach(&label, (monitor_amount - 1) / 2, 0, 2, 1);
    }

    Ok(window)
}
