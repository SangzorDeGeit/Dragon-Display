// packages for gui
use gtk::{ApplicationWindow, Box, ListBox, ListItem, ListView, ProgressBar, ResponseType, ScrolledWindow, SignalListItemFactory, SingleSelection, Stack, StringObject, TreeExpander, TreeListModel, TreeListRow};
use gtk::{Button, Label, glib, Grid, Entry, DropDown, FileChooserNative, gio};
use adw::prelude::*;
use async_channel::{Receiver, Sender};
use glib::{clone, spawn_future_local};
use tokio::time;

use std::collections::HashMap;
use std::time::Duration;
use std::{env, thread};
use std::io::{Error, ErrorKind};
use std::rc::Rc;
use std::cell::RefCell;

use super::{AddRemoveMessage, SelectMessage};
use super::config::{read_campaign_from_config, MAX_CAMPAIGN_AMOUNT, CAMPAIGN_MAX_CHAR_LENGTH, SYNCHRONIZATION_OPTIONS}; 
use super::google_drive_sync::{get_folder_all, get_folder_tree, initialize, FolderResult, InitializeMessage};
use crate::widgets::campaign_button::CampaignButton;
use crate::widgets::google_folder_object::GoogleFolderObject;
use crate::widgets::remove_button::RemoveButton;
use crate::runtime;
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
    button_connect_4_gd.connect_clicked(clone!(@strong sender, @strong app, @strong campaign_name, @strong campaign_path, @strong button_connect_4_gd => move |_| {
        let (gd_sender, gd_receiver) = async_channel::unbounded();
        button_connect_4_gd.set_sensitive(false);
        runtime().spawn(async move {
            initialize(gd_sender).await;
        });
        glib::spawn_future_local(clone!(@strong app, @strong sender, @strong campaign_name, @strong campaign_path, @strong label_4_gd => async move {
            while let Ok(message) = gd_receiver.recv().await {
                match message {
                    InitializeMessage::UserConsentUrl { url } => {
                        let updated_label = format!("In order to use the google drive synchronization service you need to give dragon display permission to connect to your google account.\n Your browser should open automatically, if it doesn't open go to the following link on your browser: {}", url);
                        label_4_gd.set_text(&updated_label);
                    }
                    InitializeMessage::Token { token } => {
                        let name = campaign_name.borrow().to_string();
                        let path_str = campaign_path.borrow().to_string();
                        let access_token = token.access_token;
                        let refresh_token = token.refresh_token;
                        let campaign = Campaign {
                            name: name,
                            path: path_str,
                            sync_option: SynchronizationOption::GoogleDrive { access_token: access_token, refresh_token: refresh_token, google_drive_sync_folder: "".to_string()},
                        };
                        select_google_drive_path(&app, campaign, sender.clone()); 
                    }
                    InitializeMessage::Error { error } => sender.send_blocking(AddRemoveMessage::Error { error: error, fatal: true }).expect("channel closed"),
                }
            }
        }));
    }));

    button_previous_4_gd.connect_clicked(clone!(@strong stack => move |_| {
        stack.set_visible_child(&page_3)
    }));

    Ok(window)
}

pub enum FolderAmount{
    Total {amount: usize},
    Current {amount : usize},
}

/// This method is part of the add campaign process. It is split from the original
/// add_campaign_window method because this page needs data (tokens) to setup the page, this is not
/// known yet when naming the campaign and selecting the option. Therefore this part is in a
/// seperate window
fn select_google_drive_path(app: &adw::Application, campaign: Campaign, sender: Sender<AddRemoveMessage>) {
    let progress_bar = ProgressBar::builder()
        .fraction(0.0)
        .show_text(true)
        .build();

    let (folder_amount_sender, folder_amount_receiver) = async_channel::unbounded();
    let mut total = 1.0;
    let mut current = 0.0;

    glib::spawn_future_local(clone!(@strong progress_bar => async move {
        while let Ok(amount) = folder_amount_receiver.recv().await {
            match amount {
                FolderAmount::Total { amount } => {
                    if amount > 0 {
                        total = amount as f64;
                    }
                }
                FolderAmount::Current { amount } => {
                    let new_current = current + amount as f64;
                    if new_current <= total {
                        current = new_current;
                    }
                }
            }
            progress_bar.set_text(Some(&format!("Loading folder: {}/{}", current, total)));
            let new_fraction = current/total;
            progress_bar.set_fraction(new_fraction);
        }
    }));

    let container = Grid::new();
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Dragon-Display")
        .modal(true)
        .deletable(false)
        .child(&container)
        .build();

    let button_cancel = Button::builder()
        .label("Cancel")
        .build();
    button_cancel.set_margin_all(6);

    let button_choose = Button::builder()
        .label("Choose")
        .sensitive(false)
        .build();
    button_choose.set_margin_all(6);

    let label = Label::builder()
        .label("Dragon display is getting the google drive data, please wait...")
        .wrap(true)
        .build();
    label.set_margin_all(6);
    
    // This variable represents the folder ID that is selected
    let current_id = Rc::new(RefCell::new(String::from("root")));
    let label_selection = Label::builder()
        .label("")
        .wrap(true)
        .build();
    label_selection.set_margin_all(6);

    container.attach(&label, 0, 0, 2, 1);
    container.attach(&label_selection, 0, 1, 2, 1);
    container.attach(&progress_bar, 0, 2, 2, 1);
    container.attach(&button_cancel, 0, 4, 1, 1);
    container.attach(&button_choose, 1, 4, 1, 1);

    let(access_token, refresh_token) = match campaign.sync_option.clone() {
        SynchronizationOption::GoogleDrive { access_token, refresh_token, .. } => (access_token, refresh_token), 
        _ => {
            sender.send_blocking(AddRemoveMessage::Error { error: Error::new(ErrorKind::InvalidInput, "Google select path was called for a none google drive sync campaign"), fatal: false })
                .expect("channel closed");
            return
        }
    };

    let new_access_token = Rc::new(RefCell::new(access_token.clone()));
    let new_refresh_token = Rc::new(RefCell::new(refresh_token.clone()));

    // We call the request for all the folders in an async thread which sends the folder_result
    // back to the main event loop
    let (foldertree_sender, foldertree_receiver) = async_channel::unbounded();

    runtime().spawn(clone!(@strong sender => async move {
        let (total, access_token, refresh_token) = match get_folder_all(access_token, refresh_token).await {
            Ok(r) => r,
            Err(e) => {
                sender.send_blocking(AddRemoveMessage::Error { error: e, fatal: false }).expect("Channel closed");
                return
            },
        };
        folder_amount_sender.send_blocking(FolderAmount::Total { amount: total }).expect("Channel Closed");

        let mut id_name_map = HashMap::new();
        id_name_map.insert("root".to_string(), "My Drive".to_string());
        let id_child_map= HashMap::new();
        let folder_result = FolderResult {
            id_name_map,
            id_child_map,
            access_token,
            refresh_token,
        };

        let result = match get_folder_tree(folder_result, "root".to_string(), folder_amount_sender).await {
            Ok(r) => r,
            Err(e) => {
                sender.send_blocking(AddRemoveMessage::Error { error: e, fatal: false }).expect("Channel closed");
                return
            },
        };
        foldertree_sender.send_blocking(result).expect("Channel closed");
    }));


    // AWait a message from the async thread that reqeusts folders
    glib::spawn_future_local(clone!(@strong current_id, @strong new_access_token, @strong new_refresh_token, @strong button_choose => async move {
        while let Ok(result) = foldertree_receiver.recv().await {
            new_access_token.replace(result.access_token);
            new_refresh_token.replace(result.refresh_token);
            // we create a liststore for our root model, this contains one element labelled My drive
            let root_folder = GoogleFolderObject::new("My Drive".to_string(), "root".to_string());
            let root_vec: Vec<GoogleFolderObject> = vec![root_folder];
            let root_store = gio::ListStore::new::<GoogleFolderObject>();
            // add the root folder (as vector) to the root_model
            root_store.extend_from_slice(&root_vec);
            // We create a TreeListModel with as root the root_store variable. Whenever an item gets
            // clicked we want present a new store based on the item that was clicked
            // This model is just to instantiate the data, it does not create any widgets
            let tree_model = TreeListModel::new(root_store, true, false,  move |item| {
                let folder_item = item.downcast_ref::<GoogleFolderObject>().expect("Found a non folder object when creating the google drive tree");
                let store = gio::ListStore::new::<GoogleFolderObject>();
                // Get all the children from the item that was clicked
                let folder_id = folder_item.id();
                let children = result.id_child_map.get(&folder_id).expect("Clicked folder id could not be found in the map");
                // Make a folder object for each child and add them to a vector
                let mut child_folder_vec = Vec::new();
                for child_id in children {
                    let child_name = result.id_name_map.get(child_id).expect("No name found for the child of clicked folder");
                    let child_folder = GoogleFolderObject::new(child_name.to_string(), child_id.to_string());
                    child_folder_vec.push(child_folder);
                }
                store.extend_from_slice(&child_folder_vec);
                Some(store.upcast::<gio::ListModel>())
            });

            // To create the widgets, we need a SignalListItemFactory
            let factory = SignalListItemFactory::new();
            // The first step in the factory is to create a new label for every widget that is requested by
            // the model. 
            factory.connect_setup(move |_, list_item| {
                let hbox = Box::new(gtk::Orientation::Horizontal, 5);
                let expander = TreeExpander::new();
                let label = Label::new(None);
                hbox.append(&expander);
                hbox.append(&label);
                list_item
                    .downcast_ref::<ListItem>()
                    .expect("item needs to be a list_item")
                    .set_child(Some(&hbox));
                });

            // We want to set the Label of the widget and we want to connect the TreeExpander to the
            // TreeListRow
            factory.connect_bind(clone!(@strong tree_model => move |_, list_item| {
                let folder_object = list_item
                    .downcast_ref::<ListItem>()
                    .expect("connect_bind in ListItemFactory: item needs to be a ListItem")
                    .item()
                    .and_downcast::<GoogleFolderObject>()
                    .expect("Connect_bind in ListItemFactory: item was not a TreeListRow");

                let position = list_item
                    .downcast_ref::<ListItem>()
                    .expect("connect_bind in ListItemFactory: item needs to be a ListItem")
                    .position();

                let hbox = list_item
                    .downcast_ref::<ListItem>()
                    .expect("connect_bind in ListItemFactory: item needs to be a ListItem")
                    .child()
                    .and_downcast::<Box>()
                    .expect("Connect_bind in ListItemFactory: child was not a Label");

                let first_child = hbox
                    .first_child()
                    .expect("Connect_bind in ListItemFactory: box did not contain first child");
                let expander = first_child
                    .downcast_ref::<TreeExpander>()
                    .expect("Connect_bind in ListItemFactory: first child was not a tree expander");

                let last_child = hbox
                    .last_child()
                    .expect("Connect_bind in ListItemFactory: box did not contain last child");
                let label = last_child
                    .downcast_ref::<Label>()
                    .expect("Connect_bind in ListItemFactory: first child was not a label");

                let tree_object = tree_model.row(position);
                expander.set_list_row(tree_object.as_ref());
                label.set_label(&folder_object.name());
            }));

            // Only allow one item to be selected
            let selection_model = SingleSelection::new(Some(tree_model));

            selection_model.connect_selection_changed(clone!(@strong current_id, @strong label_selection, @strong button_choose => move |model, _, _| {
                let position = model.selected();
                let binding = model.item(position);
                let folder_object = binding.and_downcast_ref::<GoogleFolderObject>()
                    .expect("connect_selection in selection model: item needs to be GoogleFolderObject");
                label_selection.set_label(&folder_object.name());
                current_id.replace(folder_object.id());
                button_choose.set_sensitive(true);
            }));

            let list_view = ListView::new(Some(selection_model), Some(factory));
            list_view.set_single_click_activate(false);


            let scrolled_window = ScrolledWindow::builder()
                .hscrollbar_policy(gtk::PolicyType::Never)
                .child(&list_view)
                .height_request(400)
                .build();
            
            label_selection.set_label("My Drive");
            label.set_label("Select a folder where Dragon-Display will download the images from. Current folder: ");
            container.remove(&progress_bar);
            container.attach(&scrolled_window, 0, 3, 2, 1);
        }
    }));

    button_cancel.connect_clicked(clone!(@strong sender, @strong window => move |_| {
        sender.send_blocking(AddRemoveMessage::Cancel).expect("Channel closed");
        window.close();
    }));

    button_choose.connect_clicked(clone!(@strong window, @strong campaign, @strong current_id, @strong new_access_token, @strong new_refresh_token => move |_| {
        let name = campaign.name.clone();
        let path = campaign.path.clone();
        let new_campaign = Campaign {
            name,
            path,
            sync_option: SynchronizationOption::GoogleDrive { 
                access_token: new_access_token.borrow().to_string(), 
                refresh_token: new_refresh_token.borrow().to_string(), 
                google_drive_sync_folder: current_id.borrow().to_string() 
            },
        };
        sender.send_blocking(AddRemoveMessage::Campaign { campaign: new_campaign }).expect("Channel closed");
        window.close();
    }));


    window.present();
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
