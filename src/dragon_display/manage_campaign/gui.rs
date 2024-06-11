// packages for gui
use gtk::{DialogFlags, ResponseType,ApplicationWindow, Stack};
use gtk::{Button, Label, Box, glib, Grid, Entry, DropDown, FileChooserNative, Dialog};
use adw::prelude::*;

use std::borrow::Borrow;
use std::cell::RefCell;
use std::env;
use std::fs::create_dir_all;
use std::io::{Error, ErrorKind};

use crate::dragon_display::{manage_campaign::{config::read_campaign_from_config, add_campaign, remove_campaign, MAX_CAMPAIGN_AMOUNT, CAMPAIGN_MAX_CHAR_LENGTH, SYNCHRONIZATION_OPTIONS}, 
                            google_drive_sync};
use crate::widgets::campaign_button::{self, CampaignButton};

const CAMPAIGN_NAME: &str = "CAMPAIGN_NAME";
const CAMPAIGN_PATH: &str = "CAMPAIGN_PATH";

const ALLOWED_CHARS: [char; 65] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 
    'h', 'i', 'j', 'k', 'l', 'm', 'n', 
    'o', 'p', 'q', 'r', 's', 't', 'u', 
    'v', 'w', 'x', 'y', 'z', 'A', 'B', 
    'C', 'D', 'E', 'F', 'G', 'H', 'I', 
    'J', 'K', 'L', 'M', 'N', 'O', 'P', 
    'Q', 'R', 'S', 'T', 'U', 'V', 'W', 
    'X', 'Y', 'Z', '0', '1', '2', '3', 
    '4', '5', '6', '7', '8', '9', '-', 
    '\'', ' '
];

// The "main"/"select campaign" window
pub fn select_campaign_window(app: &adw::Application) -> Result<ApplicationWindow, Error>{

    // use the settings var to store information about the gui
    // let settings = Settings::new(APP_ID);

    let container = Grid::new();
    let window = ApplicationWindow::builder()
    .application(app)
    .title("Dragon-Display")
    .child(&container)
    .build();


    let mut max_campaigns_reached: bool = false;
    let label = Label::builder()
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .wrap(true)
        .max_width_chars(40)
        .build();
    let button_add = Button::builder()
        .label("add campaign")
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();
    let button_remove = Button::builder()
        .label("remove campaign")
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();
    

    let campaign_list = read_campaign_from_config();
    //To add the campaign buttons    
    match campaign_list {
        Ok(list) => {
            label.set_text("Select a campaign");
            let mut i = 0;
            container.attach(&button_remove, i, 2, 1, 1);
            for campaign in list {
                let campaign_button = CampaignButton::new(campaign, ref_app, window);
                container.attach(&campaign_button, i, 1, 1, 1)
            }

            if i%2 == 0 {
                container.attach(&label, i/2, 0, 2, 1);
            } else {
                container.attach(&label, (i/2)+1, 0, 1, 1);
            }
            if i >= i32::from(MAX_CAMPAIGN_AMOUNT) {max_campaigns_reached = true}
            container.attach(&button_add, i+1, 2, 1, 1);

        }
        Err(e) => {
            match e.kind() {
                ErrorKind::NotFound => label.set_text("You do not have any campaigns yet"),
                ErrorKind::InvalidInput => label.set_text("An inalid operation was used, something wrong in source code"),
                ErrorKind::OutOfMemory => label.set_text(format!("You cannot have more then {} campaigns, please delete .config.toml file and restart the program (this file is a hidden file in the directory of this program", MAX_CAMPAIGN_AMOUNT).as_str()),
                _ => label.set_text("The '.config.toml' file most likely got corrupted. Please delete this file and restart the program (this file is a hidden file in the directory of this program)"),
            }
            
            container.attach(&label, 0, 0, 1, 1);
            container.attach(&button_add, 0, 1, 1, 1);
        }
    }
    container.set_halign(gtk::Align::Center);
    container.set_valign(gtk::Align::Center);
    

    button_add.connect_clicked(glib::clone!(@strong app, @strong window => move |_| {
        if max_campaigns_reached{
            create_error_dialog(&app, format!("You can only have {} campaigns at a time", MAX_CAMPAIGN_AMOUNT).as_str())
        }
        else {
            window.destroy();
            add_campaign_window(&app);
        }

    }));

    button_remove.connect_clicked(glib::clone!(@strong app, @strong window => move |_| {
        window.destroy();
        remove_campaign_window(&app);
    }));

    Ok(window)
}





fn add_campaign_window(app: &adw::Application) {
    //The stack contains the different pages to setup a new campaign
    let stack = Stack::new();
    stack.set_transition_type(gtk::StackTransitionType::SlideLeftRight);

    //create the window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Add Campaign")
        .child(&stack)
        .resizable(true)
        .default_width(400)
        .default_height(100)
        .build();

    //initialize general widgets(s)
    let button_cancel = Button::builder()
        .label("Cancel")
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();

    //initialize widgets for page 1
    let label_1 = Label::builder()
        .label("Name the campaign")
        .margin_top(6)
        .margin_bottom(6)
        .build();
    let button_next_1 = Button::builder()
        .label("Next")
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();
    let entry_1 = Entry::new();
    match env::var(CAMPAIGN_NAME){
        Ok(name) => entry_1.set_text(&name),
        Err(_) => entry_1.set_text(""),
    }
    entry_1.buffer().set_max_length(Some(CAMPAIGN_MAX_CHAR_LENGTH));

    let page_1 = Grid::new();
    page_1.attach(&label_1, 2, 1, 1, 1);
    page_1.attach(&entry_1, 2, 2, 1, 1);
    page_1.attach(&button_next_1, 3, 3, 1, 1);
    page_1.attach(&button_cancel, 1, 3, 1, 1);
    page_1.set_halign(gtk::Align::Center);
    page_1.set_valign(gtk::Align::Center);


    //initialize widgets for page 2
    let label_2 = Label::builder()
        .label("Choose synchronization service")
        .margin_top(6)
        .margin_bottom(6)
        .build();
    let button_next_2 = Button::builder()
        .label("Next")
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();
    let button_previous_2 = Button::builder()
        .label("Back")
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();
    let dropdown_2 = DropDown::from_strings(&SYNCHRONIZATION_OPTIONS);

    let page_2 = Grid::new();
    page_2.attach(&label_2, 2, 1, 1, 1);
    page_2.attach(&button_previous_2, 2, 3, 1, 1);
    page_2.attach(&button_next_2, 3, 3, 1, 1);
    page_2.attach(&button_cancel, 1, 3, 1, 1);
    page_2.attach(&dropdown_2, 2, 2, 1, 1);
    page_2.set_halign(gtk::Align::Center);
    page_2.set_valign(gtk::Align::Center);



    // initalize widgets for page 3
    match env::current_dir().unwrap().to_str() {
        Some(path) => env::set_var(CAMPAIGN_PATH, path),
        None => env::set_var(CAMPAIGN_PATH, ""),
    }
    let label_3 = Label::builder()
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .wrap(true)
        .build();
        match env::var(CAMPAIGN_PATH) {
            Ok(path) => label_3.set_text(format!("Choose location of the image folder, this the folder where all the images to be displayed by the program are stored. Current location: {}", &path).as_str()),
            Err(_) => label_3.set_text("Choose location of image folder, this the folder where all the images to be displayed by the program are stored."),
        }
    let button_default_3 = Button::builder()
        .label("Use Default")
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();    
     let button_choose_3 = Button::builder()
        .label("Choose Location")
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();
     let button_previous_3 = Button::builder()
        .label("Back")
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();
     let button_next_3 = Button::builder()
        .label("Next")
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();

    let file_chooser = FileChooserNative::new(
        Some("Choose location of image folder"),
        Some(&window),
        gtk::FileChooserAction::SelectFolder,
        Some("Select"),
        Some("Cancel")
    );

    let page_3 = Grid::new();
    page_3.attach(&label_3, 1, 1, 3, 1);
    page_3.attach(&button_previous_3, 2, 3, 1, 1);
    page_3.attach(&button_cancel, 1, 3, 1, 1);
    page_3.attach(&button_default_3, 3, 2, 1, 1);
    page_3.attach(&button_choose_3, 2, 2, 1, 1);
    page_3.attach(&button_next_3, 3, 3, 1, 1);
    page_3.set_halign(gtk::Align::Center);
    page_3.set_valign(gtk::Align::Center);




    // initalize widgets for (optional) page 4 -> Google Drive (gd)
    let label_4_gd = Label::builder()
        .label("In order to use the google drive synchronization service you need to give dragon display permission to connect to your google account.")
        .margin_top(6)
        .margin_bottom(6)
        .wrap(true)
        .build();
    let button_connect_4_gd = Button::builder()
        .label("Connect")
        .margin_bottom(6)
        .margin_end(6)
        .margin_start(6)
        .margin_top(6)
        .build();
    let button_previous_4_gd = Button::builder()
        .label("Back")
        .margin_bottom(6)
        .margin_end(6)
        .margin_start(6)
        .margin_top(6)
        .build();

    let page_4_gd = Grid::new();
    page_4_gd.attach(&label_4_gd, 0, 0, 2, 1);
    page_4_gd.attach(&button_connect_4_gd, 1, 1, 1, 1);
    page_4_gd.attach(&button_previous_4_gd, 0, 1, 1, 1);
    page_4_gd.attach(&button_cancel, 1, 1, 1, 1);
    page_4_gd.set_halign(gtk::Align::Center);
    page_4_gd.set_valign(gtk::Align::Center);



    //add all pages to the stack
    stack.add_child(&page_1);
    stack.add_child(&page_2);
    stack.add_child(&page_3);
    stack.add_child(&page_4_gd);
    stack.set_visible_child(&page_1);





    //set actions for general widget(s)
    button_cancel.connect_clicked(glib::clone!(@strong app, @strong window => move |_| {
        window.destroy();
        select_campaign_window(&app)
    }));




    //set actions for widgets of page 1
    button_next_1.connect_clicked(glib::clone!(@strong app, @strong stack, @strong page_2 => move |_| {
        let input = String::from(entry_1.text().as_str());
        match valid_name(&input) {
            Ok(_) => {
                env::set_var(CAMPAIGN_NAME, input.as_str().trim());
                stack.set_visible_child(&page_2)
            }
            Err(error) => {
                let msg = format!("Could not add campaign: {}", error.to_string());
                create_error_dialog(&app, &msg)
            }
        }
    }));




    //set actions for widgets of page 2
    button_previous_2.connect_clicked(glib::clone!(@strong stack => move |_| {
        stack.set_visible_child(&page_1);
    }));
    button_next_2.connect_clicked(glib::clone!(@strong stack, @strong page_3, @strong button_next_3, @strong dropdown_2 => move |_| {
        match dropdown_2.selected(){
            0 => {
                button_next_3.set_label("Finish")
            },
            _ => {
                button_next_3.set_label("Next")
            },
        }
        stack.set_visible_child(&page_3)
    }));

    


    //set actions for widgets of page 3
    file_chooser.connect_response(glib::clone!(@strong label_3 => move |file_chooser, response| {
        match response {
            gtk::ResponseType::Accept => {
                match file_chooser.file() {
                    Some(f) => {
                        label_3.set_text(format!("Choose location of the image folder, use default will create a new dedicated folder in the current directory. Current location: {}", f.path().unwrap().to_str().unwrap()).as_str());
                        env::set_var(CAMPAIGN_PATH, f.path().unwrap().to_str().unwrap())
                    },
                    None => {}
                }
            },
            _ => {}
        }
    }));
    button_choose_3.connect_clicked(glib::clone!(@strong file_chooser => move |_| file_chooser.set_visible(true)));

    button_default_3.connect_clicked(glib::clone!(@strong app, @strong window, @strong stack, @strong dropdown_2, @strong page_4_gd => move |_| {
        match env::current_dir().unwrap().to_str() {
            Some(path) => {
                match env::var(CAMPAIGN_NAME) {
                    Ok(name) => {
                        let completepath = path.to_string()+"/"+&name;
                        env::set_var(CAMPAIGN_PATH, &completepath)

                    },
                    Err(_) => create_error_dialog(&app, "Could not find campaign name to create folder")

                }
                label_3.set_text(format!("Choose location of the image folder. Current location: {}", path).as_str());
            },
            None => create_error_dialog(&app, "could not find the default directory"),
        }
        match dropdown_2.selected(){
            0 => {
                match env::var(CAMPAIGN_PATH) {
                    Ok(path) => {
                        add_campaign(&app, &path, None, None, SYNCHRONIZATION_OPTIONS[0]);
                        window.destroy();
                    },
                    Err(_) => create_error_dialog(&app, "Select a location for the image folder")
                };
            },
            _ => stack.set_visible_child(&page_4_gd),
        }
    }));

    button_previous_3.connect_clicked(glib::clone!(@strong stack, @strong page_2 =>move |_| {
        stack.set_visible_child(&page_2)
    }));

    button_next_3.connect_clicked(glib::clone!(@strong app, @strong window, @strong stack => move |_| {
        match env::var(CAMPAIGN_PATH) {
            Ok(path) => {
                match valid_folder(&path){
                    Ok(_) => {
                        match dropdown_2.selected(){
                            0 => {
                                add_campaign(&app, &path, None, None, SYNCHRONIZATION_OPTIONS[0]);
                                window.destroy();
                            },
                            _ => {
                                stack.set_visible_child(&page_4_gd)
                            },
                       } 
                    },
                    Err(_) => create_error_dialog(&app, "This location is already used by another campaign")
                }  
            }
            Err(_) => create_error_dialog(&app, "Select a location for the image folder")
        };
       
    }));




    //set actions for widgets of optional page 4 -> Google Drive (gd)
    button_connect_4_gd.connect_clicked(glib::clone!(@strong window, @strong app => move |_| {
        match google_drive_sync::initialize() {
            Ok(t) => {
                let access_token = t.access_token;
                let refresh_token = t.refresh_token;
                match env::var(CAMPAIGN_PATH) {
                    Ok(path) => add_campaign(&app, &path, Some(access_token), Some(refresh_token), SYNCHRONIZATION_OPTIONS[1]),
                    Err(_) => create_error_dialog(&app, "Select a location for the image folder")
                };
                
                window.destroy();
            },
            Err(error) => {
                match error.kind() {
                    ErrorKind::NotFound => {
                        let msg = format!("Could not synchronize, you are missing a client_secret.json in the Dragon-Display directory");
                        create_error_dialog(&app, &msg.as_str());
                        select_campaign_window(&app)
                    }
                    _ => {
                        let msg = format!("Could not add campaign: {}", error.to_string());
                        create_error_dialog(&app, &msg.as_str());
                        select_campaign_window(&app)
                    }
                }
            }
        }
    }));
    button_previous_4_gd.connect_clicked(glib::clone!(@strong stack => move |_| {
        stack.set_visible_child(&page_3)
    }));


    window.present();

}






// Checks for valid campaign name
fn valid_name(name: &str) -> Result<(), Error> {
    let trimmed_name = name.trim();

    if trimmed_name.chars().all(char::is_whitespace) {
        return Err(Error::from(ErrorKind::InvalidInput))
    }

    if !trimmed_name.chars().all(|x| ALLOWED_CHARS.contains(&x)) {
        println!("not valid name");
        return Err(Error::from(ErrorKind::InvalidInput))
    }
    println!("valid name");

    let campaign_list = match read_campaign_from_config() {
        Ok(c) => c,
        Err(_) => return Ok(())
    };

    for campaign in campaign_list {
        if campaign.0 == trimmed_name {
            return Err(Error::from(ErrorKind::AlreadyExists))
        }
    }

    Ok(())
}






fn valid_folder(path: &str) -> Result<(), Error> {
    let campaign_list = read_campaign_from_config()?;
    for campaign in campaign_list {
        if campaign.1.path == path {
            return Err(Error::from(ErrorKind::AlreadyExists))
        }
    }

    Ok(())
}






// The 'remove campaign' window
fn remove_campaign_window(app: &adw::Application){
    let campaign_list = read_campaign_from_config();

    let label = Label::builder()
        .label("Select the campaign you want to remove")
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();
    let button_cancel = Button::builder()
        .label("Cancel")
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();


    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Dragon-Display")
        .resizable(false)
        .build();

    let container = Grid::new();
    match campaign_list {
        Ok(list) => {
            let mut i = 0;
            for campaign in list {
                let campaign_button = Button::builder()
                    .label(&campaign.0)
                    .margin_top(6)
                    .margin_bottom(6)
                    .margin_start(6)
                    .margin_end(6)
                    .build();
                let campaign_name = campaign.0.clone();
                let campaign_path = campaign.1.path.clone();
                campaign_button.connect_clicked(glib::clone!(@strong app, @strong window, @strong campaign_name, @strong campaign_path =>move |_| {
                    let confirm_window = remove_campaign_window_confirm(&window, campaign_name.as_str());
                    confirm_window.present();
                    confirm_window.connect_response(glib::clone!(@strong app, @strong confirm_window, @strong campaign_name, @strong campaign_path, @strong window => move |_, response| {
                        match response {
                            ResponseType::Yes => {
                                remove_campaign(&app, campaign_name.as_str(), campaign_path.as_str());
                                confirm_window.destroy();
                                window.destroy()
                            },
                            _ => confirm_window.destroy()
                        }
                    }));
                }));
                container.attach(&campaign_button, i, 1, 1, 1);
                i += 1;
            }
            i -= 1;
            if i%2 == 0 {
                container.attach(&label, i/2, 0, 1, 1);
                container.attach(&button_cancel, i/2, 2, 1, 1);
            } else {
                container.attach(&label, i/2, 0, 2, 1);
                container.attach(&button_cancel, i/2, 2, 2, 1);
            }
        }
        Err(_) => {
            create_error_dialog(app, "There are no campaigns to remove!");
        }
    }

    window.set_content(Some(&container));

    button_cancel.connect_clicked(glib::clone!(@strong app, @strong window => move |_| {
        window.destroy();
        select_campaign_window(&app)
    }));

    window.present();
}





// A modal window to confirm the removal of a campaign
fn remove_campaign_window_confirm(window: &adw::ApplicationWindow, campaign_name: &str) -> Dialog {
    let msg = format!("delete {}?", campaign_name);
    let buttons = [("On second though, No", ResponseType::No), ("Do it!", ResponseType::Yes)];
    let window = Dialog::with_buttons(Some(msg), Some(window), DialogFlags::MODAL, &buttons);
    return window;
}




// function to create an error dialog
pub fn create_error_dialog(app: &adw::Application, msg: &str) {
    let label = Label::builder()
        .label(msg)
        .margin_bottom(6)
        .margin_top(6)
        .margin_start(6)
        .margin_end(6)
        .build();
    let button = Button::builder()
        .label("Ok")
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();

    let container = Box::new(gtk::Orientation::Vertical, 10);
    container.append(&label);
    container.append(&button);

    let window = Dialog::builder()
        .application(app)
        .child(&container)
        .resizable(false)
        .build();
    window.set_modal(true);

    button.connect_clicked(glib::clone!(@strong window => move |_| {
        window.destroy()
    }));

    window.present()
}




    