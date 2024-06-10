use gdk4::prelude::*;
use gtk::prelude::{ButtonExt, GridExt, GtkWindowExt};
use gtk::{Button, Label, glib, Grid, ApplicationWindow};

use crate::dragon_display::manage_campaign::config::Campaign;


pub fn select_monitor_window(app: &adw::Application, campaign: Box<Campaign>) {
    let container = Grid::new();
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Dragon-Display")
        .child(&container)
        .build();

    let label = Label::builder()
        .label("Choose the screen that you want to display the images on")
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();

    match gdk4::Display::default() {
        Some(display) => {
            let mut i: u32 = 0;
            while display.monitors().item(i).is_some() {
                let monitor = display
                    .monitors()
                    .item(i)
                    .unwrap()
                    .to_value()
                    .get::<gdk4::Monitor>()
                    .expect("The value needs to be monitor!");
                
                let monitor_button = Button::builder()
                    .label(format!("{}cm x {}cm", monitor.height_mm()/10, monitor.width_mm()/10).as_str())
                    .margin_top(6)
                    .margin_bottom(6)
                    .margin_start(6)
                    .margin_end(6)
                    .build();

                monitor_button.connect_clicked(glib::clone!(@strong window => move |_| {
                    window.destroy()
                }));

                container.attach(&monitor_button, i32::try_from(i).ok().unwrap(), 1, 1, 1);
                i = i+1;
            }
            let i = i32::try_from(i).ok().unwrap();
            container.attach(&label, ((i/2)+(i%2))-1, 0, 2-(i%2), 1)
        },
        None => todo!()
    }

    window.present();
}

fn control_panel(app: &adw::Application) {
    todo!()
}

fn image_display(app: &adw::Application) {
    todo!()
}


// let container = Grid::new();
// let window = ApplicationWindow::builder()
// .application(app)
// .title("Dragon-Display")
// .child(&container)
// .build();

// let label = Label::builder()
// .label("Choose the screen that you want to display the images on")
// .margin_top(6)
// .margin_bottom(6)
// .margin_start(6)
// .margin_end(6)
// .build();

// match gdk4::Display::default() {"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",
//     Some(display) => {
//         let mut i = 0;
//         while display.monitors().item(i).is_some() {
//             let monitor = display
//             .monitors()
//             .item(i)
//             .unwrap()
//             .to_value()
//             .get::<gdk4::Monitor>()
//             .expect("The value needs to be monitor!");
            
//             let monitor_button = Button::builder()
//             .label(format!("{}mm x {}mm", monitor.height_mm(), monitor.width_mm()).as_str())
//             .margin_top(6)
//             .margin_bottom(6)
//             .margin_start(6)
//             .margin_end(6)
//             .build();

//             monitor_button.connect_clicked(glib::clone!(@strong window => move |_| {
//                 window.destroy()
//             }));

//             container.attach(&monitor_button, i32::try_from(i).ok().unwrap(), 1, 1, 1);
//             i = i+1;
//         }
//         let i = i32::try_from(i).ok().unwrap();
//         container.attach(&label, ((i/2)+(i%2))-1, 0, 2-(i%2), 1)
//     },
//     None => todo!()
// }
// window.present()