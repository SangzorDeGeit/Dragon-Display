#![feature(proc_macro_hygiene, decl_macro)]

//Packages for configuration file
use std::fs::File;
use std::io::{Read, ErrorKind, Write};
use serde::{Deserialize, Serialize};
use toml::to_string;
use tokio;

//Packages for GUI
use druid::widget::{prelude::*, ViewSwitcher, RadioGroup};
use druid::widget::{Flex, Label, Button};
use druid::{commands, AppLauncher, Data, UnitPoint, WidgetExt, WindowDesc, FileDialogOptions, AppDelegate, DelegateCtx, Target, Command, Handled, Lens};

//packages for cloud syncing

//imported modules
pub mod campaigns;
pub mod google_drive_sync;

const SYNCHRONIZATION_OPTIONS: [(&str, SynchronizationOptions); 3]= [
    ("None", SynchronizationOptions::None),
    ("Google Drive", SynchronizationOptions::GoogleDrive),
    ("OneDrive", SynchronizationOptions::OneDrive)
];

//add all the types of supported cloud services for this program
#[derive(PartialEq, Clone, Data, Debug, Deserialize, Serialize)]
enum SynchronizationOptions {
    None,
    GoogleDrive,
    OneDrive,
}

#[derive(Clone, Data, Debug, Lens)]
struct ConfigurationGUI{
    configuration: Configuration,
    page_number: u8
}

#[derive(Clone, Data, Debug, Deserialize, Serialize, Lens)]
struct Configuration {
    synchronization_option: SynchronizationOptions,
    image_directory: Option<String>,
}

#[derive(Clone, Data)]
struct Campaign {
    name: String
}
struct Delegate;

fn config_gui() -> impl Widget<ConfigurationGUI> {
    let configuration_page = ViewSwitcher::new(
        |data: &ConfigurationGUI, _env: &Env| data.clone(), 
        |selector, data, _env| match selector.page_number {
            1 => 
            match &data.configuration.image_directory {
                Some(image_directory) => {Box::new(
                    Flex::column()
                        .with_child(Label::new(format!("Image directory: {}", image_directory)))
                        .with_spacer(20.0)
                        .with_child(Button::new("Select folder").on_click(move |ctx, _data: &mut ConfigurationGUI, _| 
                            {ctx.submit_command(commands::SHOW_OPEN_PANEL.with(
                            FileDialogOptions::new()
                            .select_directories()
                            .name_label("Target")
                            .title("Select folder with display images")
                            .button_text("Select")
                        ))}))
                        .with_spacer(20.0)
                        .with_child(Button::new("Next").on_click(|_ctx, data: &mut ConfigurationGUI, _| data.page_number+=1))
                )},
                None => {
                    Box::new(
                        Flex::column()
                            .with_child(Label::new("Select image folder"))
                            .with_spacer(20.0)
                            .with_child(Button::new("Select folder").on_click(move |ctx, _data: &mut ConfigurationGUI, _| 
                                {ctx.submit_command(commands::SHOW_OPEN_PANEL.with(
                                FileDialogOptions::new()
                                .select_directories()
                                .name_label("Target")
                                .title("Select folder with display images")
                                .button_text("Select")
                            ))}))
                    )
                }
            },
            _ => Box::new(
                    Flex::column()
                        .with_child(Label::new("Choose synchronization service"))
                        .with_spacer(20.0)
                        .with_child(RadioGroup::column(SYNCHRONIZATION_OPTIONS.to_vec()).lens(Configuration::synchronization_option).lens(ConfigurationGUI::configuration))
                        .with_spacer(20.0)
                        .with_child(Button::new("Finish").on_click(move |ctx, data: &mut ConfigurationGUI, _| {                        
                            let mut configuration_file = File::create("config.toml").expect("coult not create file");
                            let toml_string = to_string(&data.configuration).expect("Could not convert configuration into string");
                            configuration_file.write_all(toml_string.as_bytes()).expect("could not write to file");
                            ctx.submit_command(commands::CLOSE_WINDOW)
                        }))
                    )
        }
    );
    Flex::row()
        .with_child(configuration_page)
        .align_vertical(UnitPoint::CENTER)
}


fn config_make() {
    let configuration_item = Configuration {
        synchronization_option: SynchronizationOptions::None,
        image_directory: None
    };
   
    let configuration_gui = ConfigurationGUI {
        configuration: configuration_item,
        page_number: 1
    };
    let configuration_screen = WindowDesc::new(config_gui())
        .title("Configurations")
        .window_size((400.0, 200.0));
    AppLauncher::with_window(configuration_screen)
        .delegate(Delegate)
        .log_to_console()
        .launch(configuration_gui)
        .expect("Failed to start application");

}

fn config_read() -> Configuration {
    let mut configuration_file = File::open("config.toml").unwrap_or_else(|error| {
        if error.kind() == ErrorKind::NotFound {
            config_make();
            todo!()
        } else {
            panic!("Could not open file: {:?}", error);
        }
    });
    todo!();
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

#[tokio::main]
async fn main() {
    println!("Start");
    let access_token = google_drive_sync::initialize().await;
    println!("{}",access_token.refresh_token);
    //google_drive_sync::something();
    println!("Done")
    // config_read();
    // //Describe main window
    // let campaign_selection = WindowDesc::new(choose_campaign_widget())
    //     .title("Dragon-Display")
    //     .window_size((100.0, 100.0));

    // let option: Campaign = Campaign { 
    //     name: "Uclia".into(),
    //  };
    // //Start the application
    // AppLauncher::with_window(campaign_selection)
    //     .log_to_console()
    //     .launch(option)
    //     .expect("Failed to start application");
}


impl AppDelegate<ConfigurationGUI> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut ConfigurationGUI,
        _env: &Env
    ) -> Handled {
        if let Some(file_info) = cmd.get(commands::OPEN_FILE) {
                if let Some(path) = file_info.path().to_str() {
                    data.configuration.image_directory = Option::from(String::from(path.to_string()));
                    return Handled::Yes;
                }
        }
        Handled::No
    }
}
