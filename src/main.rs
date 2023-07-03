//Packages for configuration file
use std::fs::File;
use std::io::{Read, ErrorKind};
use druid::lens::Unit;
use druid::piet::cairo::glib::FileError;
use toml;
use serde::Deserialize;

//Packages for GUI
use druid::widget::prelude::*;
use druid::widget::{Flex, Label, Button};
use druid::{commands, AppLauncher, Data, UnitPoint, WidgetExt, WindowDesc, FileDialogOptions, AppDelegate, DelegateCtx, Target, Command, Handled};



//add all the types of supported cloud services for this program
#[derive(Debug, Deserialize)]
enum rclone {
    GoogleDrive,
    OneDrive,
}

#[derive(Debug, Deserialize)]
struct Configuration {
    rclone: Option<rclone>,
    sync_directory: Option<String>,
    image_directory: String,
    default_screen: i32
}

#[derive(Clone, Data)]
struct Campaign {
    name: String
}

struct Delegate;

fn config_gui() -> impl Widget<u8> {
    // let message = Label::new("Choose if you want to synchronize images via rclone. \n 
    // Choose none if you do not want to use the synchronization feature.");
    let select_dialog_options = FileDialogOptions::new()
        .select_directories()
        .name_label("Target")
        .title("Select folder with display images")
        .button_text("Select");
    let mut message = Label::new("Choose the location of the images \nImages in this folder can be displayed by the program");
    let select = Button::new("Select folder").on_click(move |ctx, data, _| 
        {ctx.submit_command(commands::SHOW_OPEN_PANEL.with(select_dialog_options.clone()))});

    Flex::column()
        .with_child(message)
        .with_spacer(20.0)
        .with_child(select)
        .align_vertical(UnitPoint::CENTER)
}


fn config() -> File {
    let config_screen = WindowDesc::new(config_gui())
        .title("Configurations")
        .window_size((200.0, 200.0));

    AppLauncher::with_window(config_screen)
        .delegate(Delegate)
        .log_to_console()
        .launch(1.to_owned())
        .expect("Failed to start application");

    File::create("config.toml").expect("Could not create file")
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

fn start_program(data: Campaign) {
    todo!()
}

fn choose_campaign_widget() -> impl Widget<Campaign> {
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
        .window_size((100.0, 100.0));

    let option: Campaign = Campaign { 
        name: "Uclia".into(),
     };
    //Start the application
    AppLauncher::with_window(campaign_selection)
        .log_to_console()
        .launch(option)
        .expect("Failed to start application");
}

impl AppDelegate<u8> for Delegate {
    fn command(
        &mut self,
        ctx: &mut DelegateCtx,
        target: Target,
        cmd: &Command,
        data: &mut u8,
        env: &Env
    ) -> Handled {
        if let Some(file_info) = cmd.get(commands::OPEN_FILE) {
            
        }
    }
}
