use adw::Application;
use async_channel::Sender;
use gtk::prelude::ObjectExt;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{gio, glib};
use std::env;
use anyhow::{bail, Context, Result};

use crate::config::read_campaign_from_config;
use crate::setup_manager::AddRemoveMessage;

mod imp {
    use std::cell::RefCell;
    use std::env;
    use std::io::{Error, ErrorKind};
    use std::rc::Rc;

    use super::{valid_name, valid_path};
    use async_channel::Sender;
    use glib::subclass::InitializingObject;
    use gtk::subclass::prelude::*;
    use gtk::{
        glib, template_callbacks, Button, CompositeTemplate, DropDown, Entry, FileChooserDialog,
        Label, Stack,
    };
    use gtk::{prelude::*, ResponseType};

    use crate::config::{Campaign, SynchronizationOption};
    use crate::setup_manager::AddRemoveMessage;

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dragon/display/add_campaign.ui")]
    pub struct AddCampaignWindow {
        #[template_child]
        pub stack: TemplateChild<Stack>,
        #[template_child]
        pub entry: TemplateChild<Entry>,
        #[template_child]
        pub dropdown: TemplateChild<DropDown>,
        #[template_child]
        pub location_label: TemplateChild<Label>,
        #[template_child]
        pub finish_button: TemplateChild<Button>,
        pub campaign_name: RefCell<String>,
        pub path: Rc<RefCell<String>>,
        pub sync_option: RefCell<SynchronizationOption>,
        pub sender: RefCell<Option<Sender<AddRemoveMessage>>>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for AddCampaignWindow {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "DdAddCampaignWindow";
        type Type = super::AddCampaignWindow;
        type ParentType = gtk::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Button::ensure_type();

            klass.bind_template();
            klass.bind_template_callbacks()
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[template_callbacks]
    impl AddCampaignWindow {
        #[template_callback]
        fn handle_cancel(&self, _: Button) {
            self.sender
                .borrow()
                .clone()
                .expect("No sender found")
                .send_blocking(AddRemoveMessage::Cancel)
                .expect("Channel closed");
        }

        #[template_callback]
        fn handle_back(&self, _: Button) {
            let current_name = self
                .stack
                .visible_child_name()
                .expect("No current visible page")
                .to_string();
            match current_name.as_str() {
                "page2" => self.stack.set_visible_child_name("page1"),
                "page3" => self.stack.set_visible_child_name("page2"),
                _ => panic!("Found a back signal on the first page"),
            }
        }

        #[template_callback]
        fn handle_next_page1(&self, _: Button) {
            let input = self.entry.text().to_string();
            match valid_name(&input) {
                Ok(_) => {
                    self.campaign_name.replace(input);
                    self.stack.set_visible_child_name("page2");
                }
                Err(e) => {
                    self.sender
                        .borrow()
                        .clone()
                        .expect("No sender found")
                        .send_blocking(AddRemoveMessage::Error {
                            error: e,
                            fatal: false,
                        })
                        .expect("Channel closed");
                    return;
                }
            }
        }

        #[template_callback]
        fn handle_next_page2(&self, _: Button) {
            let sync_option = self.dropdown.selected();
            match sync_option {
                0 => {
                    self.sync_option.replace(SynchronizationOption::None);
                    self.finish_button.set_label("Finish");
                }
                _ => {
                    self.sync_option
                        .replace(SynchronizationOption::GoogleDrive {
                            access_token: "".to_string(),
                            refresh_token: "".to_string(),
                            google_drive_sync_folder: "".to_string(),
                        });
                    self.finish_button.set_label("Next");
                }
            }
            self.stack.set_visible_child_name("page3");
        }

        #[template_callback]
        fn handle_choose(&self, b: Button) {
            let binding = b.root().expect("No root found");
            let window = binding
                .downcast_ref::<gtk::Window>()
                .expect("Root could not be downcasted to window");
            let file_chooser = FileChooserDialog::new(
                Some("Choose location of image folder"),
                Some(window),
                gtk::FileChooserAction::SelectFolder,
                &[
                    ("Select", ResponseType::Accept),
                    ("Cancel", ResponseType::Cancel),
                ],
            );
            let campaign_path = self.path.clone();
            let location_label = self.location_label.clone();
            file_chooser.connect_response(move |file_chooser, response| {
                match response {
                    ResponseType::Accept => (),
                    ResponseType::Cancel => {
                        file_chooser.close();
                        return;
                    }
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
                let msg = format!("Current location: {}", path_str);
                location_label.set_label(&msg);
                campaign_path.replace(path_str);
                file_chooser.close();
            });
            file_chooser.set_visible(true);
        }

        #[template_callback]
        fn handle_default(&self, button: Button) {
            let sender = self.sender.borrow().clone().expect("No sender found");
            let working_dir = match env::current_dir() {
                Ok(d) => d,
                Err(_) => {
                    sender
                        .send_blocking(AddRemoveMessage::Error {
                            error: Error::new(
                                ErrorKind::PermissionDenied, 
                                "Could not find current directory, please try to run again as administrator"
                                ).into(),
                                fatal: false,
                        })
                        .expect("Channel closed");
                    return;
                }
            };
            let working_dir = match working_dir.to_str() {
                Some(d) => d,
                None => {
                    sender
                        .send_blocking(AddRemoveMessage::Error {
                            error: Error::new(
                                ErrorKind::NotFound,
                                "Could find the current directory",
                            ).into(),
                            fatal: false,
                        })
                        .expect("Channel closed");
                    return;
                }
            };

            let default_path = format!("{}/{}", working_dir, self.entry.text().to_string());  
            match valid_path(&default_path) {
                Ok(_) => (),
                Err(e) => {
                    sender.send_blocking(AddRemoveMessage::Error { error: e, fatal: false })
                        .expect("Channel closed");
                    return;
                },
            }
            self.path.replace(default_path);
            self.handle_finish(button);
        }

        #[template_callback]
        fn handle_finish(&self, _: Button) {
            let sender = self.sender.borrow().clone().expect("No sender found");
            let path = self.path.borrow().to_string();
            match valid_path(&path) {
                Ok(_) => (),
                Err(e) => {
                    sender.send_blocking(AddRemoveMessage::Error { 
                        error: e, fatal: false })
                        .expect("Channel closed");
                    return;
                },
            }
            let campaign = match self.dropdown.selected() {
                0 => Campaign { 
                    name: self.entry.text().to_string(), 
                    path, 
                    sync_option: SynchronizationOption::None },
                1 => Campaign { 
                    name: self.entry.text().to_string(), 
                    path, 
                    sync_option: SynchronizationOption::GoogleDrive { 
                        access_token: "".to_string(), 
                        refresh_token: "".to_string(), 
                        google_drive_sync_folder: "".to_string() 
                    }
                },
                _ => panic!("An invalid choice was made"),
            };
            sender.send_blocking(AddRemoveMessage::Campaign { campaign }).expect("Channel closed");
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for AddCampaignWindow {
        fn constructed(&self) {
            // Call "constructed" on parent
            self.parent_constructed();
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for AddCampaignWindow {}

    // Trait shared by all windows
    impl WindowImpl for AddCampaignWindow {}

    // Trait shared by all application windows
    impl ApplicationWindowImpl for AddCampaignWindow {}
}

glib::wrapper! {
    pub struct AddCampaignWindow(ObjectSubclass<imp::AddCampaignWindow>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
            @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                        gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl AddCampaignWindow {
    pub fn new(app: &Application, sender: Option<Sender<AddRemoveMessage>>) -> Self {
        // set all properties
        let object = glib::Object::new::<Self>();
        let imp = object.imp();
        imp.sender.replace(sender);
        object.set_property("application", app);
        object
    }
}

const ALLOWED_CHARS: [char; 66] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
    'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3', '4',
    '5', '6', '7', '8', '9', '-', '\'', ' ', '_',
];

// Validate user input function for the campaign name
pub fn valid_name(name: &str) -> Result<()> {
    let trimmed_name = name.trim();

    if trimmed_name.chars().all(char::is_whitespace) {
        bail!("Input may not be all whitespace")     }

    if !trimmed_name.chars().all(|x| ALLOWED_CHARS.contains(&x)) {
        bail!("Input contained invalid character")
    }

    let campaign_list = read_campaign_from_config()?;
    if campaign_list.is_empty() {
        return Ok(());
    }

    for campaign in campaign_list {
        if campaign.name == trimmed_name {
            bail!("Name already exists");
        }
    }

    Ok(())
}

pub fn valid_path(path: &str) -> Result<()> {
    let campaign_list = read_campaign_from_config()?;
    for campaign in campaign_list {
        if campaign.path == path {
            bail!("Another campaign already uses this folder")
        }
    }
    let current_dir = env::current_dir()?;
    let current_dir_str = current_dir.to_str().context("Could not convert current directory to a string")?;
    if path == current_dir_str {
        bail!("Cannot use the current working directory as a folder for campaign images")
    }

    Ok(())
}
