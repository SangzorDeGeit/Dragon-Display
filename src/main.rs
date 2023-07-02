//Packages for configuration file
use std::fs::File;
use std::io::{Read, ErrorKind};
use druid::piet::cairo::glib::FileError;
use toml;
use serde::Deserialize;

//Packages for GUI
use druid::widget::prelude::*;
use druid::widget::{Flex, Label, Button};
use druid::{AppLauncher, Data, UnitPoint, WidgetExt, WindowDesc};



//add all the types of supported cloud services for this program
#[derive(Debug, Deserialize)]
enum rclone {
    GoogleDrive,
    OneDrive,
}

#[derive(Debug, Deserialize)]
struct Configuration {
    rclone: Option<rclone>,
    syncDirectory: Option<String>,
    imageDirectory: String,
    defaultScreen: i32
}

#[derive(Clone, Data)]
struct Campaign {
    name: String
}

fn configGUI() {

    todo!();
}

fn config() -> File {
    File::create("config.toml");

   todo!();
}

fn read_config(){
    let mut file = File::open("config.toml").unwrap_or_else(|error| {
        if error.kind() == ErrorKind::NotFound {
            config()
        } else {
            panic!("Could not open file: {:?}", error);
        }
    });
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read file");

    println!("{}", contents);
}


fn choose_campaign_widget() -> impl Widget<Campaign>{
    let message = Label::new("Choose campaign:");
    let dummy_campaign = Button::new(|data: &Campaign, _env: &Env| data.name.to_string());

    Flex::column()
        .with_child(message)
        .with_spacer(20.0)
        .with_child(dummy_campaign)
        .align_vertical(UnitPoint::CENTER)

}
fn main() {
    read_config();
    //Describe main window
    let campaign_selection = WindowDesc::new(choose_campaign_widget())
        .title("Dragon-Display")
        .window_size((400.0, 400.0));

    let option: Campaign = Campaign { 
        name: "Uclia".into(),
     };
    //Start the application
    AppLauncher::with_window(campaign_selection)
        .log_to_console()
        .launch(option)
        .expect("Failed to start application");
}
