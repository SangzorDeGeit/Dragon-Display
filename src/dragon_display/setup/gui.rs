// packages for gui
use gtk::{ApplicationWindow, ListBox, ResponseType, ScrolledWindow, Stack};
use gtk::{Button, Label, glib, Grid, Entry, DropDown, FileChooserNative, gio};
use adw::prelude::*;
use async_channel::{Receiver, Sender};
use glib::{clone, spawn_future_local};

use std::env;
use std::io::{Error, ErrorKind};
use std::rc::Rc;
use std::cell::RefCell;

use super::{AddRemoveMessage, SelectMessage};
use super::config::{read_campaign_from_config, MAX_CAMPAIGN_AMOUNT, CAMPAIGN_MAX_CHAR_LENGTH, SYNCHRONIZATION_OPTIONS}; 
use super::google_drive_sync::{InitializeMessage, initialize};
use crate::widgets::campaign_button::CampaignButton;
use crate::widgets::remove_button::RemoveButton;
use super::{Campaign, SynchronizationOption};


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

trait CustomMargin {
    fn set_margin_all(&self, margin: i32);
}

impl CustomMargin for Button {
    fn set_margin_all(&self, margin: i32) {
        self.set_margin_end(margin);
        self.set_margin_start(margin);
        self.set_margin_top(margin);
        self.set_margin_bottom(margin);
    }
}

impl CustomMargin for Label {
    fn set_margin_all(&self, margin: i32) {
        self.set_margin_end(margin);
        self.set_margin_start(margin);
        self.set_margin_top(margin);
        self.set_margin_bottom(margin);
    }
}

// The "main"/"select campaign" window
pub fn select_campaign_window(app: &adw::Application, sender: Sender<SelectMessage>) -> Result<ApplicationWindow, Error> {

    let container = Grid::new();
    container.set_hexpand(true);
    container.set_vexpand(true);
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

    let button_add = Button::builder()
        .label("add campaign")
        .build();
    button_add.set_margin_all(6);

    let button_remove = Button::builder()
        .label("remove campaign")
        .build();
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
    if i%2 == 0 {
        container.attach(&label, i/2, 0, 2, 1);
    } else {
        container.attach(&label, (i/2)+1, 0, 1, 1);
    }

    container.attach(&button_add, i+1, 2, 1, 1);

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




    // initalize widgets for (optional) page 4 -> Google Drive (gd)
    let label_4_gd = Label::builder()
        .label("In order to use the google drive synchronization service you need to give dragon display permission to connect to your google account.")
        .wrap(true)
        .build();
    label_4_gd.set_margin_all(6);

    let button_connect_4_gd = Button::builder()
        .label("Connect")
        .build();
    button_connect_4_gd.set_margin_all(6);

    let button_previous_4_gd = Button::builder()
        .label("Back")
        .build();
    button_previous_4_gd.set_margin_all(6);

    let page_4_gd = Grid::new();
    page_4_gd.attach(&label_4_gd, 0, 0, 2, 1);
    page_4_gd.attach(&button_connect_4_gd, 1, 1, 1, 1);
    page_4_gd.attach(&button_previous_4_gd, 0, 1, 1, 1);
    page_4_gd.set_halign(gtk::Align::Center);
    page_4_gd.set_valign(gtk::Align::Center);


    //add all pages to the stack
    stack.add_child(&page_1);
    stack.add_child(&page_2);
    stack.add_child(&page_3);
    stack.add_child(&page_4_gd);
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
        println!("{}", campaign_path.borrow());
    }));

    button_choose_3.connect_clicked(clone!(@strong file_chooser => move |_| file_chooser.set_visible(true)));

    button_default_3.connect_clicked(clone!(@strong stack, @strong dropdown_2, @strong page_4_gd, @strong campaign_name, @strong campaign_path, @strong sender => move |_| {
        let name = campaign_name.borrow().to_string();
        let path_str = format!("{}/{}", &working_dir, name);
        campaign_path.replace(path_str.clone()); 
        label_3.set_text(format!("Choose location of the image folder. Current location: {}", path_str).as_str());

        match dropdown_2.selected(){
            0 => {
                let campaign = Campaign {
                    name: name,
                    path: path_str,
                    sync_option: SynchronizationOption::None,
                };
                sender.clone().send_blocking(AddRemoveMessage::Campaign { campaign: campaign}).expect("Channel closed");
            },
            _ => stack.set_visible_child(&page_4_gd),
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
        match dropdown_2.selected() {
            0 => {
                let campaign = Campaign {
                    name: name,
                    path: path_str,
                    sync_option: SynchronizationOption::None,
                };
                sender.clone().send_blocking(AddRemoveMessage::Campaign { campaign: campaign }).expect("Channel closed");
            }
            _ => {
                stack.set_visible_child(&page_4_gd);
            }
        }
    }));

    button_cancel_3.connect_clicked(clone!(@strong sender => move |_| {
        sender.send_blocking(AddRemoveMessage::Cancel).expect("Channel closed");
        sender.close();
    }));



    //set actions for widgets of optional page 4 -> Google Drive (gd)
    button_connect_4_gd.connect_clicked(clone!(@strong sender, @strong app => move |_| {
        let (gd_sender, gd_receiver) = async_channel::unbounded();
        button_connect_4_gd.set_sensitive(false);
        gio::spawn_blocking(move || {
            initialize(gd_sender);
        });
        glib::spawn_future_local( async move {
            while let Ok(message) = gd_receiver.recv().await {
                match message {
                    InitializeMessage::UserConsentUrl { url } => todo!("update the label"),
                    InitializeMessage::Token { token } => {

                    }
                    InitializeMessage::Error { error } => todo!("send error to manager"),
                }
            }
        });
        // match initialize(gd_sender) {
        //     Ok(t) => {
        //         let path_str = campaign_path.borrow().to_string();
        //         let name = campaign_name.borrow().to_string();
        //         let access_token = t.access_token;
        //         let refresh_token = t.refresh_token;
        //         let campaign = Campaign {
        //             name: name,
        //             path: path_str,
        //             sync_option: SynchronizationOption::GoogleDrive { access_token: access_token, refresh_token: refresh_token },
        //         };
        //         sender.clone().send_blocking(AddRemoveMessage::Campaign { campaign: campaign }).expect("Channel closed");
        //     },
        //     Err(e) => {
        //         sender.clone().send_blocking(AddRemoveMessage::Error { error: e, fatal: false }).expect("Channel closed");
        //     }
        // }
    }));

    button_previous_4_gd.connect_clicked(clone!(@strong stack => move |_| {
        stack.set_visible_child(&page_3)
    }));

    Ok(window)
}






/// Checks for valid campaign name and makes error dialog if not
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





/// Checks if the folder to be created already exists, or is not the current working directory 
/// Creates an error dialog if the path is invalid
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



// The 'remove campaign' window
pub fn remove_campaign_window(app: &adw::Application, sender: Sender<AddRemoveMessage>) -> Result<ApplicationWindow, Error> {
    let campaign_list = read_campaign_from_config()?;
    if campaign_list.len() == 0 {
        sender.send_blocking(AddRemoveMessage::Cancel).expect("Channel closed");
    }

    let label = Label::builder()
        .label("Select the campaign you want to remove")
        .build();
    label.set_margin_all(6);
    
    let button_cancel = Button::builder()
        .label("Cancel")
        .build();
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
    i =- 1;

    if i%2 == 0 {
        container.attach(&label, i/2, 0, 1, 1);
        container.attach(&button_cancel, i/2, 2, 1, 1);
    } else {
        container.attach(&label, i/2, 0, 2, 1);
        container.attach(&button_cancel, i/2, 2, 2, 1);
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


fn remove_campaign_confirm(app: &adw::Application, campaign: Campaign, sender: Sender<AddRemoveMessage>) {
    let message = format!("Are you sure you want to delete {}?", campaign.name.as_str());
    let label = Label::builder()
        .label(message)
        .build();
    label.set_margin_all(6);
    
    let button_yes = Button::builder()
        .label("Yes")
        .build();
    button_yes.set_margin_all(6);
    
    let button_no = Button::builder()
        .label("No")
        .build();
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

/// The campaign's google drive path may be empty
pub fn select_google_drive_path(app: &adw::Application, campaign: Campaign) -> ApplicationWindow {
    // Maybe a short loading screen while the folders are being requested

    let container = Grid::new();
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Dragon-Display")
        .child(&container)
        .build();

    let button_cancel = Button::builder()
        .label("Cancel")
        .build();
    button_cancel.set_margin_all(6);

    let button_choose = Button::builder()
        .label("Choose")
        .build();
    button_choose.set_margin_all(6);

    let label = Button::builder()
        .label("Select a folder where Dragon-Display will download the images from")
        .build();
    label.set_margin_all(6);
    
    let label_path = Button::builder()
        .label("Current path: 'root'")
        .build();
    label_path.set_margin_all(6);
    
    let drives_box = ListBox::new();
    let scrollwindow = ScrolledWindow::builder()
        .child(&drives_box)
        .build();
    // ListboxRow is the widget that can be set as children for the listbox
    // Every time a row is clicked it will make a request for the folders in that row
    // Make new listbox with children
    // detach old listbox and attach new one
    window
}

pub fn select_monitor_window(app: &adw::Application) {
    let container = Grid::new();
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Dragon-Display")
        .child(&container)
        .build();

    let label = Label::builder()
        .label("Choose the screen that you want to display the images on")
        .build();
    label.set_margin_all(6);

    let display = match gdk4::Display::default() {
        Some(d) => d,
        None => todo!(),
    };

    let mut i: u32 = 0;
    while let Some(monitor) = display.monitors().item(i) {
        let monitor = monitor.to_value()
            .get::<gdk4::Monitor>()
            .expect("Value needs to be monitor");

        let monitor_button = Button::builder()
            .label(format!("{}cm x {}cm", monitor.height_mm()/10, monitor.width_mm()/10).as_str())
            .build();
        monitor_button.set_margin_all(6);

        monitor_button.connect_clicked(clone!(@strong window => move |_| {
            window.destroy();
            todo!("Send the monitor to the manager");
        }));

        let column = match i32::try_from(i) {
            Ok(c) => c,
            Err(_) => todo!("break here and display an error message (too many monitors)"),
        };
        container.attach(&monitor_button, column, 1, 1, 1);

        i = i+1;
    }
    let monitor_amount = match i32::try_from(i) {
        Ok(c) => c,
        Err(_) => todo!("break here and display an error message (too many monitors)"),
    };
    if monitor_amount == 0 {
        label.set_text("Could not detect any monitors");
        container.attach(&label, 0, 0, 0, 0);
    } else if monitor_amount%2 == 1 {
        container.attach(&label, monitor_amount/2, 0, 1, 1);
    } else {
        container.attach(&label, (monitor_amount-1)/2, 0, 2, 1);
    }

    window.present();
}
