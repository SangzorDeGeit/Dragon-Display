use gtk::glib::{clone, spawn_future_local};
use gtk::prelude::*;
use gtk::{glib, gio, ApplicationWindow, Button, Grid, Label, ScrolledWindow, ListView, SingleSelection, ListItem, TreeExpander, SignalListItemFactory, LinkButton, TreeListModel, ProgressBar, Box};

use super::CustomMargin;
use crate::dragon_display::setup::config::{Campaign, SynchronizationOption};
use crate::dragon_display::setup::google_drive::{get_folder_amount, get_folder_tree, initialize_client, synchronize_files, FolderResult, InitializeMessage};
use crate::dragon_display::setup::AddRemoveMessage;
use crate::widgets::google_folder_object::GoogleFolderObject;
use crate::runtime;

use async_channel::Sender;
use std::io::{Error, ErrorKind};
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

// The frontend directly comminucates with the backend to reduce the amount of channels created. The manager simply gets a signal from the
// frontend when the adding process fails or succeeeds
pub enum GdBackendToFrondend {
    URL { url: String },
    Folders,
}

pub enum TreeWidgetMessage {
    ProgressBar {progressbar: ProgressBar},
    FolderTree {foldertree: ScrolledWindow, access_token: String, refresh_token: String},
    FolderSelection {folder_name: String, folder_id: String},
}

pub enum FolderAmount{
    Total {amount: usize},
    Current {amount : usize},
}

pub fn googledrive_connect_window(
    app: &adw::Application,
    campaign: Campaign,
    sender: Sender<AddRemoveMessage>,
    reconnect: bool,
) -> Result<ApplicationWindow, Error> {
    // ui elements
    let message: &str;
    if reconnect {
        message = "Google Drive session is expired, please reconnect to continue using google drive synchronization.";
    } else {
        message = "In order to use Google Drive synchronization you need to give Dragon-Display permission to connect to your Google Account";
    }

    let label = Label::builder().label(message).wrap(true).build();
    label.set_margin_all(6);

    let button_connect = Button::builder().label("connect").build();
    button_connect.set_margin_all(6);

    let button_cancel = Button::builder().label("cancel").build();
    button_cancel.set_margin_all(6);

    let container = Grid::new();
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Dragon-Display")
        .modal(true)
        .deletable(false)
        .child(&container)
        .build();

    container.attach(&label, 0, 0, 2, 1);
    container.attach(&button_connect, 1, 2, 1, 1);
    container.attach(&button_cancel, 0, 2, 1, 1);

    // ui logic
    let google_drive_folder = match campaign.clone().sync_option {
        SynchronizationOption::GoogleDrive { google_drive_sync_folder: gd_sync_folder, ..} => gd_sync_folder,
        _ => return Err(Error::new(ErrorKind::InvalidInput, "A non-google-drive campaign has been inputted to a function for google-drive synchronization campaigns (googledrive_connect_window)")),
    };

    let (server_terminator_sender, server_terminator_receiver) = async_channel::bounded(1);

    button_cancel.connect_clicked(clone!(@strong sender => move |_| {
        server_terminator_sender.send_blocking(()).expect("Channel closed");
        sender.send_blocking(AddRemoveMessage::Cancel).expect("Channel closed");
    }));

    // The frontend (button) will directly call an initialize_client function instead of signaling to the
    // manager to reduce the amount of channels needed
    let (gd_sender, gd_receiver) = async_channel::unbounded();
    button_connect.connect_clicked(clone!(@weak button_connect => move |_| {
        runtime().spawn(clone!(@strong gd_sender, @strong server_terminator_receiver => async move {
            initialize_client(gd_sender, server_terminator_receiver).await;
        }));
        button_connect.set_sensitive(false);
    }));

    // Wait for the initialize function to return data
    spawn_future_local(clone!(@strong google_drive_folder => async move {
        while let Ok(m) = gd_receiver.recv().await {
            match m {
                InitializeMessage::UserConsentUrl { url } => {
                    let new_message = format!(
                        "{}, if it does not open click the following link:",
                        &message
                    );
                    label.set_text(&new_message);
                    let link = LinkButton::builder()
                        .uri(url)
                        .label("Authentication Link")
                        .margin_end(6)
                        .margin_top(6)
                        .margin_start(6)
                        .margin_bottom(6)
                        .build();
                    container.attach(&link, 0, 1, 2, 1);
                }
                InitializeMessage::Token { token } => {
                    let new_campaign = Campaign {
                        name: campaign.clone().name,
                        path: campaign.clone().path,
                        sync_option: SynchronizationOption::GoogleDrive {
                            access_token: token.access_token,
                            refresh_token: token.refresh_token,
                            google_drive_sync_folder: google_drive_folder.clone(),
                        },
                    };
                    sender
                        .send_blocking(AddRemoveMessage::Campaign {
                            campaign: new_campaign,
                        })
                        .expect("Channel closed");
                    button_connect.set_sensitive(true);
                }
                InitializeMessage::Error { error: e } => {
                    sender
                        .send_blocking(AddRemoveMessage::Error {
                            error: e,
                            fatal: false,
                        })
                        .expect("Channel closed");
                }
            }
        }
    }));

    Ok(window)
}

/// Create the window to select a folder in googledrive where the images get downloaded from
pub fn googledrive_select_path_window(
    app: &adw::Application,
    campaign: Campaign,
    sender: Sender<AddRemoveMessage>,
) -> Result<ApplicationWindow, Error> {
    // ui elements
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

    let button_refresh = Button::builder()
        .label("Refresh")
        .sensitive(false)
        .build();
    button_refresh.set_margin_all(6);

    let label = Label::builder()
        .label("Dragon display is getting the google drive data, please wait...")
        .wrap(true)
        .build();
    label.set_margin_all(6);
    
    // This variable represents the folder ID that is selected by the user
    let current_id = Rc::new(RefCell::new(String::from("root")));
    let label_selection = Label::builder()
        .label("")
        .wrap(true)
        .build();
    label_selection.set_margin_all(6);

    container.attach(&label, 0, 0, 2, 1);
    container.attach(&label_selection, 0, 1, 2, 1);
    container.attach(&button_cancel, 0, 4, 1, 1);
    container.attach(&button_refresh, 0, 3, 2, 1);
    container.attach(&button_choose, 1, 4, 1, 1);

    // ui logic
    label_selection.set_label("My Drive");
    label.set_label("Select a folder where Dragon-Display will download the images from. Current folder: ");

    let(access_token, refresh_token) = match campaign.sync_option.clone() {
        SynchronizationOption::GoogleDrive { access_token, refresh_token, .. } => (access_token, refresh_token), 
        _ => return Err(Error::new(ErrorKind::InvalidInput, "Google select path was called for a none google drive sync campaign (googledrive_select_path_window)")),
    };

    let (widget_sender, widget_receiver) = async_channel::unbounded::<TreeWidgetMessage>();
    let new_access_token = Rc::new(RefCell::new(access_token.clone()));
    let new_refresh_token = Rc::new(RefCell::new(refresh_token.clone()));

    create_tree_widget(access_token, refresh_token, widget_sender.clone(), sender.clone());
    button_refresh.set_sensitive(false);
    // await messages from the tree widget creator
    spawn_future_local(clone!(@strong current_id, @strong new_access_token, @strong new_refresh_token, @weak button_refresh, @weak button_choose => async move {
        while let Ok(message) = widget_receiver.recv().await {
            match message {
                TreeWidgetMessage::ProgressBar { progressbar } => {
                    if let Some(widget) = container.child_at(0, 2) {
                        container.remove(&widget);
                    }
                    container.attach(&progressbar, 0, 2, 2, 1);
                },
                TreeWidgetMessage::FolderTree { foldertree, access_token, refresh_token } => {
                    new_access_token.replace(access_token);
                    new_refresh_token.replace(refresh_token);
                    if let Some(widget) = container.child_at(0, 2) {
                        container.remove(&widget);
                    }
                    container.attach(&foldertree, 0, 2, 2, 1);
                    button_refresh.set_sensitive(true);
                },
                TreeWidgetMessage::FolderSelection {folder_name, folder_id } => {
                    label_selection.set_label(&folder_name);
                    current_id.replace(folder_id);
                    button_choose.set_sensitive(true);
                },
            } 
        };
    }));

    button_cancel.connect_clicked(clone!(@strong sender => move |_| {
        sender.send_blocking(AddRemoveMessage::Cancel).expect("Channel closed");
    }));

    button_choose.connect_clicked(clone!(@strong campaign, @strong current_id, @strong new_access_token, @strong new_refresh_token, @strong sender => move |_| {
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
    }));

    button_refresh.connect_clicked(clone!(@weak button_refresh => move |_| {
        create_tree_widget(new_access_token.borrow().to_string(), new_refresh_token.borrow().to_string(), widget_sender.clone(), sender.clone());
        button_refresh.set_sensitive(false);
        button_choose.set_sensitive(false);
    }));

    Ok(window)
}

/// Folder tree widgets should be made seperately - we have the window that sets up the basic
/// buttons.  
/// The progressBar followed by the scrollWindow containing the TreeList should be generated in a seperate
/// function.
fn create_tree_widget(access_token: String, refresh_token: String, widget_sender: Sender<TreeWidgetMessage>, sender: Sender<AddRemoveMessage>) { 
    let progress_bar = ProgressBar::builder()
        .fraction(0.0)
        .show_text(true)
        .build();

    let (update_progressbar_sender, update_progressbar_receiver) = async_channel::unbounded();
    let mut total = 1.0;
    let mut current = 0.0;

    let new_access_token = Rc::new(RefCell::new(access_token.clone()));
    let new_refresh_token = Rc::new(RefCell::new(refresh_token.clone()));

    // Update the progress bar
    spawn_future_local(clone!(@strong progress_bar, @strong widget_sender => async move {
        while let Ok(amount) = update_progressbar_receiver.recv().await {
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
            progress_bar.set_text(Some(&format!("Creating folder tree: {}/{}", current, total)));
            let new_fraction = current/total;
            progress_bar.set_fraction(new_fraction);
            widget_sender.send_blocking(TreeWidgetMessage::ProgressBar { progressbar: progress_bar.clone() }).expect("Channel Closed");
        }
    }));
    
    let (foldertree_sender, foldertree_receiver) = async_channel::unbounded::<FolderResult>();

    // Await a message from the async thread that reqeusts folders
    spawn_future_local(clone!(@strong new_access_token, @strong new_refresh_token, @strong widget_sender => async move {
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

            // Binding for when the selected folder is changed
            selection_model.connect_selection_changed(clone!(@strong widget_sender => move |model, _, _| {
                let position = model.selected();
                let binding = model.item(position);
                let folder_object = binding.and_downcast_ref::<GoogleFolderObject>()
                    .expect("connect_selection in selection model: item needs to be GoogleFolderObject");
                widget_sender.send_blocking(TreeWidgetMessage::FolderSelection {folder_name: folder_object.name(), folder_id: folder_object.id() }).expect("Channel closed");
            }));

            let list_view = ListView::new(Some(selection_model), Some(factory));
            list_view.set_single_click_activate(false);

            let scrolled_window = ScrolledWindow::builder()
                .hscrollbar_policy(gtk::PolicyType::Never)
                .child(&list_view)
                .height_request(400)
                .build();
            widget_sender.send_blocking(TreeWidgetMessage::FolderTree { foldertree: scrolled_window, access_token: new_access_token.borrow().to_string(), refresh_token: new_refresh_token.borrow().to_string() }).expect("Channel closed");
        }}));

    // Call the request for all the folders in an async thread, which sends the folder_result
    // back to the main event loop
    runtime().spawn(async move {
        let (total, access_token, refresh_token) = match get_folder_amount(access_token, refresh_token).await {
            Ok(r) => r,
            Err(e) => {
                sender.send_blocking(AddRemoveMessage::Error { error: e, fatal: false }).expect("Channel closed");
                return
            },
        };
        update_progressbar_sender.send_blocking(FolderAmount::Total { amount: total }).expect("Channel Closed");

        let mut id_name_map = HashMap::new();
        id_name_map.insert("root".to_string(), "My Drive".to_string());
        let id_child_map= HashMap::new();
        let folder_result = FolderResult {
            id_name_map,
            id_child_map,
            access_token,
            refresh_token,
        };

        let result = match get_folder_tree(folder_result, "root".to_string(), update_progressbar_sender).await {
            Ok(r) => r,
            Err(e) => {
                sender.send_blocking(AddRemoveMessage::Error { error: e, fatal: false }).expect("Channel closed");
                return
            },
        };
        foldertree_sender.send_blocking(result).expect("Channel closed");
    });
}

pub fn googledrive_synchronize_window(app: &adw::Application, campaign: Campaign, sender: Sender<Result<(Campaign, Vec<String>), Error>>) -> ApplicationWindow {
    // ui element
    let container = Box::new(gtk::Orientation::Vertical, 6);

    let progressbar = ProgressBar::builder()
        .fraction(0.0)
        .show_text(true)
        .build();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Dragon-Display")
        .child(&container)
        .build();

    container.append(&progressbar);

    // ui logic 
    let (sync_sender, sync_receiver) = async_channel::bounded::<(Campaign, Vec<String>)>(1);
    let (update_progress_sender, update_progress_receiver) = async_channel::unbounded();

    spawn_future_local(clone!(@strong sender, @strong campaign => async move {
        while let Ok(message) = sync_receiver.recv().await {
            sender.send_blocking(Ok((message.0, message.1))).expect("Channel closed");
        }
    }));

    let mut total = 1.0;
    let mut current = 0.0;

    spawn_future_local(async move {
        while let Ok(message) = update_progress_receiver.recv().await {
            match message {
                FolderAmount::Total { amount } => {
                    if amount > 0 {
                        total = amount as f64;
                    }
                },
                FolderAmount::Current { amount } => {
                    let new_current = current + amount as f64;
                    if  new_current <= total {
                        current = new_current;
                    } else {
                        current = total;
                    }
                },
            }
            progressbar.set_text(Some(&format!("Downloading files: {}/{}", current, total)));
            let new_fraction = current/total;
            progressbar.set_fraction(new_fraction);
        }
    });

    runtime().spawn(async move {
        let (campaign, failed_files) = match synchronize_files(campaign, update_progress_sender).await {
            Ok((c, f)) => (c, f),
            Err(e) => {
                sender.send_blocking(Err(e)).expect("Channel closed");
                return;
            },
        };

        sync_sender.send_blocking((campaign, failed_files)).expect("Channel closed");
    });

    window
} 
