use super::CustomMargin;
use crate::dragon_display::setup::AddRemoveMessage;
use crate::dragon_display::setup::config::{read_campaign_from_config, CAMPAIGN_MAX_CHAR_LENGTH, SYNCHRONIZATION_OPTIONS, Campaign, SynchronizationOption};

use gtk::prelude::*;
use gtk::{Stack, ApplicationWindow, Label, Button, Entry, Grid, FileChooserNative, DropDown, ResponseType};
use gtk::glib::clone;

use async_channel::Sender;
use std::io::{Error, ErrorKind};
use std::rc::Rc;
use std::cell::RefCell;
use std::env;

const ALLOWED_CHARS: [char; 66] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 
    'h', 'i', 'j', 'k', 'l', 'm', 'n', 
    'o', 'p', 'q', 'r', 's', 't', 'u', 
    'v', 'w', 'x', 'y', 'z', 'A', 'B', 
    'C', 'D', 'E', 'F', 'G', 'H', 'I', 
    'J', 'K', 'L', 'M', 'N', 'O', 'P', 
    'Q', 'R', 'S', 'T', 'U', 'V', 'W', 
    'X', 'Y', 'Z', '0', '1', '2', '3', 
    '4', '5', '6', '7', '8', '9', '-', 
    '\'', ' ', '_'
];

pub fn add_campaign_window(app: &adw::Application, sender: Sender<AddRemoveMessage>) -> Result<ApplicationWindow, Error>{
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

    //initialize widgets for page 1 -- input name
    let label_1 = Label::builder()
        .label("Name the campaign")
        .build();
    label_1.set_margin_all(6);

    let button_next_1 = Button::builder()
        .label("Next")
        .build();
    button_next_1.set_margin_all(6);

    let button_cancel_1 = Button::builder()
        .label("Cancel")
        .build();
    button_cancel_1.set_margin_all(6);

    let entry_1 = Entry::new();
    let campaign_name = Rc::new(RefCell::new("".to_owned()));
    entry_1.buffer().set_max_length(Some(CAMPAIGN_MAX_CHAR_LENGTH));

    let page_1 = Grid::new();
    page_1.attach(&label_1, 2, 1, 1, 1);
    page_1.attach(&entry_1, 2, 2, 1, 1);
    page_1.attach(&button_next_1, 3, 3, 1, 1);
    page_1.attach(&button_cancel_1, 1, 3, 1, 1);
    page_1.set_halign(gtk::Align::Center);
    page_1.set_valign(gtk::Align::Center);


    //initialize widgets for page 2 -- Choose the sync service
    let label_2 = Label::builder()
        .label("Choose synchronization service")
        .build();
    label_2.set_margin_all(6);

    let button_next_2 = Button::builder()
        .label("Next")
        .build();
    button_next_2.set_margin_all(6);

    let button_previous_2 = Button::builder()
        .label("Back")
        .build();
    button_previous_2.set_margin_all(6);

    let button_cancel_2 = Button::builder()
        .label("Cancel")
        .build();
    button_cancel_2.set_margin_all(6);

    let dropdown_2 = DropDown::from_strings(&SYNCHRONIZATION_OPTIONS);

    let page_2 = Grid::new();
    page_2.attach(&label_2, 2, 1, 1, 1);
    page_2.attach(&button_previous_2, 2, 3, 1, 1);
    page_2.attach(&button_next_2, 3, 3, 1, 1);
    page_2.attach(&button_cancel_2, 1, 3, 1, 1);
    page_2.attach(&dropdown_2, 2, 2, 1, 1);
    page_2.set_halign(gtk::Align::Center);
    page_2.set_valign(gtk::Align::Center);

    let working_dir = match env::current_dir()?.to_str() {
        Some(p) => p.to_string(),
        None => return Err(Error::new(ErrorKind::PermissionDenied, "Could not get the current working directory due to insufficient permissions, try to run the program as administrator")),
    };
    let campaign_path = Rc::new(RefCell::new(working_dir.clone()));
    
    // initalize widgets for page 3 -- Choose location of the image folder
    let label_3 = Label::builder()
        .wrap(true)
        .build();
        label_3.set_text("Choose location of image folder, this is the folder where all the images to be displayed by the program are stored.");
    label_3.set_margin_all(6);

    let button_default_3 = Button::builder()
        .label("Use Default")
        .build();    
    button_default_3.set_margin_all(6);

    let button_choose_3 = Button::builder()
        .label("Choose Location")
        .build();
    button_choose_3.set_margin_all(6);

     let button_previous_3 = Button::builder()
        .label("Back")
        .build();
    button_previous_3.set_margin_all(6);

    let button_next_3 = Button::builder()
        .label("Next")
        .build();
    button_next_3.set_margin_all(6);

    let button_cancel_3 = Button::builder()
        .label("Cancel")
        .build();
    button_cancel_3.set_margin_all(6);

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
    page_3.attach(&button_cancel_3, 1, 3, 1, 1);
    page_3.attach(&button_default_3, 3, 2, 1, 1);
    page_3.attach(&button_choose_3, 2, 2, 1, 1);
    page_3.attach(&button_next_3, 3, 3, 1, 1);
    page_3.set_halign(gtk::Align::Center);
    page_3.set_valign(gtk::Align::Center);

    //add all pages to the stack
    stack.add_child(&page_1);
    stack.add_child(&page_2);
    stack.add_child(&page_3);
    stack.set_visible_child(&page_1);


    //set actions for widgets of page 1
    button_next_1.connect_clicked(clone!(@strong app, @strong stack, @strong page_2, @strong sender, @strong campaign_name => move |_| {
        let entry_text = entry_1.text().to_string();
        match valid_name(&entry_text) {
            Ok(_) => {
                campaign_name.replace(entry_text);
                stack.set_visible_child(&page_2);
            },
            Err(e) => sender.send_blocking(AddRemoveMessage::Error { error: e, fatal: false }).expect("Channel closed"),
        }
    }));
    
    button_cancel_1.connect_clicked(clone!(@strong sender => move |_| {
        sender.send_blocking(AddRemoveMessage::Cancel).expect("Channel closed");
        sender.close();
    }));




    //set actions for widgets of page 2
    button_previous_2.connect_clicked(clone!(@strong stack => move |_| {
        stack.set_visible_child(&page_1);
    }));

    button_next_2.connect_clicked(clone!(@strong stack, @strong page_3, @strong button_next_3, @strong dropdown_2 => move |_| {
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
    
    button_cancel_2.connect_clicked(clone!(@strong sender => move |_| {
        sender.send_blocking(AddRemoveMessage::Cancel).expect("Channel closed");
        sender.close();
    }));




    //set actions for widgets of page 3
    file_chooser.connect_response(clone!(@strong label_3, @strong campaign_path => move |file_chooser, response| {
        // We want the name of the file that is chosen, if we cannot figure out the name of the
        // file we should do nothing
        match response {
            ResponseType::Accept => (),
            _ => return,
        }
        let folder = match file_chooser.file() {
            Some(f) => f,
            None => return,
        }; 

        let path = match folder.path() {
            Some(p) => p,
            None => return,
        };

        let path_str = match path.to_str() {
            Some(s) => s.to_string(),
            None => return,
        };

        label_3.set_text(&format!("Choose location of the image folder, use default will create a new dedicated folder in the current directory. Current location: {}", path_str));
        campaign_path.replace(path_str);
    }));

    button_choose_3.connect_clicked(clone!(@strong file_chooser => move |_| file_chooser.set_visible(true)));

    button_default_3.connect_clicked(clone!(@strong stack, @strong dropdown_2, @strong campaign_name, @strong campaign_path, @strong sender => move |_| {
        let name = campaign_name.borrow().to_string();
        let path_str = format!("{}/{}", &working_dir, name);
        campaign_path.replace(path_str.clone()); 
        label_3.set_text(format!("Choose location of the image folder. Current location: {}", path_str).as_str());

        match dropdown_2.selected(){
            // Normal campaign is selected
            0 => {
                let campaign = Campaign {
                    name: name,
                    path: path_str,
                    sync_option: SynchronizationOption::None,
                };
                sender.clone().send_blocking(AddRemoveMessage::Campaign { campaign: campaign}).expect("Channel closed");
            },
            // Google Drive campaign is selected
            _ => { 
                let campaign = Campaign {
                    name: name,
                    path: path_str,
                    sync_option: SynchronizationOption::GoogleDrive { 
                        access_token: "".to_string(), 
                        refresh_token: "".to_string(), 
                        google_drive_sync_folder: "".to_string() },
                };
                sender.clone().send_blocking(AddRemoveMessage::Campaign { campaign: campaign }).expect("Channel closed");
            },
        }
    }));

    button_previous_3.connect_clicked(clone!(@strong stack, @strong page_2 =>move |_| {
        stack.set_visible_child(&page_2)
    }));

    button_next_3.connect_clicked(clone!(@strong app, @strong stack, @strong campaign_name, @strong campaign_path, @strong sender => move |_| {
        let path_str = campaign_path.borrow().to_string();
        let name = campaign_name.borrow().to_string();
        if let Err(e) = valid_path(&path_str){
                sender.clone().send_blocking(AddRemoveMessage::Error { error: e, fatal: false }).expect("Channel closed");
                return;
        }
        match dropdown_2.selected(){
            // Normal campaign is selected
            0 => {
                let campaign = Campaign {
                    name: name,
                    path: path_str,
                    sync_option: SynchronizationOption::None,
                };
                sender.clone().send_blocking(AddRemoveMessage::Campaign { campaign: campaign}).expect("Channel closed");
            },
            // Google Drive campaign is selected
            _ => { 
                let campaign = Campaign {
                    name: name,
                    path: path_str,
                    sync_option: SynchronizationOption::GoogleDrive { 
                        access_token: "".to_string(), 
                        refresh_token: "".to_string(), 
                        google_drive_sync_folder: "".to_string() },
                };
                sender.clone().send_blocking(AddRemoveMessage::Campaign { campaign: campaign }).expect("Channel closed");
            },
        }
    }));

    button_cancel_3.connect_clicked(clone!(@strong sender => move |_| {
        sender.send_blocking(AddRemoveMessage::Cancel).expect("Channel closed");
        sender.close();
    }));

    Ok(window)
}

// Validate user input function for the campaign name
fn valid_name(name: &str) -> Result<(), Error> {
    let trimmed_name = name.trim();

    if trimmed_name.chars().all(char::is_whitespace) {
        return Err(Error::new(ErrorKind::InvalidInput, "Input may not be all whitespace"))
    }

    if !trimmed_name.chars().all(|x| ALLOWED_CHARS.contains(&x)) {
        return Err(Error::new(ErrorKind::InvalidInput, "Input contained invalid character(s)"))
    }

    let campaign_list = read_campaign_from_config()?; 
    if campaign_list.is_empty() {
        return Ok(());
    }

    for campaign in campaign_list {
        if campaign.name == trimmed_name {
            return Err(Error::new(ErrorKind::InvalidInput, "name already exists"))
        }
    }

    Ok(())
}

// Validate user input function for the selected path
fn valid_path(path: &str) -> Result<(), Error> {
    let campaign_list = read_campaign_from_config()?;
    for campaign in campaign_list {
        if campaign.path == path {
            return Err(Error::new(ErrorKind::AlreadyExists, "Another campaign already uses this folder"))
        }
    }

    let current_dir = match env::current_dir() {
        Ok(d) => d,
        Err(_) => return Ok(()),
    };

    let current_dir_str = match current_dir.to_str() {
        Some(s) => s,
        None => return Ok(()),
    };
    
    if path == current_dir_str {
        return Err(Error::new(ErrorKind::InvalidInput, "Cannot use the current working directory as a folder for campaign images"))
    }

    Ok(())
}
