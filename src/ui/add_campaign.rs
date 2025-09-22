use adw::Application;
use gtk::prelude::ObjectExt;
use gtk::{gio, glib};
use snafu::{ensure, OptionExt as _, Report, ResultExt};
use std::env;

use crate::campaign::DdCampaign;
use crate::config::read_campaign_from_config;
use crate::errors::{DragonDisplayError, IOSnafu, InvalidNameSnafu, InvalidPathSnafu, OtherSnafu};

mod imp {
    use std::cell::RefCell;
    use std::env;
    use std::rc::Rc;
    use std::sync::OnceLock;

    use super::{valid_name, valid_path};
    use glib::subclass::InitializingObject;
    use gtk::glib::subclass::Signal;
    use gtk::subclass::prelude::*;
    use gtk::{
        glib, template_callbacks, Button, CompositeTemplate, DropDown, Entry, FileChooserDialog,
        Label, Stack,
    };
    use gtk::{prelude::*, ResponseType};
    use snafu::{OptionExt, ResultExt};

    use crate::campaign::DdCampaign;
    use crate::config::SynchronizationOption;
    use crate::errors::{IOSnafu, OtherSnafu};
    use crate::try_emit;

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
            let obj = self.obj();
            obj.emit_by_name::<()>("cancel", &[]);
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
            let obj = self.obj();
            try_emit!(obj, valid_name(&input), false);
            self.campaign_name.replace(input);
            self.stack.set_visible_child_name("page2");
        }

        #[template_callback]
        fn handle_next_page2(&self, _: Button) {
            let sync_option = self.dropdown.selected();
            match sync_option {
                0 => {
                    self.finish_button.set_label("Finish");
                }
                _ => {
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
            let obj = self.obj();
            let working_dir = try_emit!(
                obj,
                env::current_dir().context(IOSnafu {
                    msg: "Could not get current working directory".to_owned()
                }),
                true
            );
            let working_dir = try_emit!(
                obj,
                working_dir.to_str().context(OtherSnafu {
                    msg: { "Could not get current working directory".to_owned() }
                }),
                true
            );

            let default_path = format!("{}/{}", working_dir, self.entry.text().to_string());
            try_emit!(obj, valid_path(&default_path), false);
            self.path.replace(default_path);
            self.handle_finish(button);
        }

        #[template_callback]
        fn handle_finish(&self, _: Button) {
            let name = self.entry.text().to_string();
            let path = self.path.borrow().to_string();
            let obj = self.obj();
            try_emit!(obj, valid_path(&path), false);
            match self.dropdown.selected() {
                0 => {
                    let campaign = DdCampaign::new(name, path, SynchronizationOption::None);
                    obj.emit_by_name::<()>("campaign-none", &[&campaign]);
                }
                1 => {
                    let sync_option = SynchronizationOption::GoogleDrive {
                        access_token: "".to_string(),
                        refresh_token: "".to_string(),
                        google_drive_sync_folder: "".to_string(),
                    };
                    let campaign = DdCampaign::new(name, path, sync_option);
                    obj.emit_by_name::<()>("campaign-gd", &[&campaign]);
                }
                _ => panic!("An invalid choice was made"),
            };
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for AddCampaignWindow {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder("cancel").build(),
                    Signal::builder("campaign-gd")
                        .param_types([DdCampaign::static_type()])
                        .build(),
                    Signal::builder("campaign-none")
                        .param_types([DdCampaign::static_type()])
                        .build(),
                    Signal::builder("error")
                        .param_types([String::static_type(), bool::static_type()])
                        .build(),
                ]
            })
        }
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
    pub fn new(app: &Application) -> Self {
        // set all properties
        let object = glib::Object::new::<Self>();
        object.set_property("application", app);
        object
    }

    /// Emit an error message based on the input error
    pub fn emit_error(&self, err: DragonDisplayError, fatal: bool) {
        let msg = Report::from_error(err).to_string();
        self.emit_by_name::<()>("error", &[&msg, &fatal]);
    }

    /// Signal emitted when the cancel button is clicked
    pub fn connect_cancel<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "cancel",
            true,
            glib::closure_local!(|window| {
                f(window);
            }),
        )
    }

    /// Signal emitted when a google drive campaign is added
    pub fn connect_campaign_gd<F: Fn(&Self, DdCampaign) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            "campaign-gd",
            true,
            glib::closure_local!(|window, campaign| {
                f(window, campaign);
            }),
        )
    }

    /// Signal emitted when a no-sync campaign is added
    pub fn connect_campaign_none<F: Fn(&Self, DdCampaign) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            "campaign-none",
            true,
            glib::closure_local!(|window, campaign| {
                f(window, campaign);
            }),
        )
    }

    /// Signal emitted when an error occures
    pub fn connect_error<F: Fn(&Self, String, bool) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            "error",
            true,
            glib::closure_local!(|window, msg, fatal| {
                f(window, msg, fatal);
            }),
        )
    }
}

const ALLOWED_CHARS: [char; 66] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
    'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3', '4',
    '5', '6', '7', '8', '9', '-', '\'', ' ', '_',
];

// Validate user input function for the campaign name
pub fn valid_name(name: &str) -> Result<(), DragonDisplayError> {
    let trimmed_name = name.trim();

    ensure!(
        !trimmed_name.chars().all(char::is_whitespace),
        InvalidNameSnafu {
            msg: "input may not be all whitespace".to_owned()
        }
    );

    ensure!(
        trimmed_name.chars().all(|x| ALLOWED_CHARS.contains(&x)),
        InvalidNameSnafu {
            msg: "Input contained invalid character".to_owned()
        }
    );

    let campaign_list = read_campaign_from_config()?;
    if campaign_list.is_empty() {
        return Ok(());
    }

    for campaign in campaign_list {
        ensure!(
            campaign.name != trimmed_name,
            InvalidNameSnafu {
                msg: "Name already exists"
            }
        );
    }

    Ok(())
}

pub fn valid_path(path: &str) -> Result<(), DragonDisplayError> {
    let campaign_list = read_campaign_from_config()?;
    for campaign in campaign_list {
        ensure!(
            campaign.path != path,
            InvalidPathSnafu {
                msg: "Another campaign already uses this folder".to_owned()
            }
        );
    }
    let current_dir = env::current_dir().context(IOSnafu {
        msg: "Could not get current working directory".to_owned(),
    })?;
    let current_dir_str = current_dir.to_str().context(OtherSnafu {
        msg: "Could not get current working directory".to_owned(),
    })?;
    ensure!(
        path != current_dir_str,
        InvalidPathSnafu {
            msg: "Cannot use the current working directory as a folder for campaign images"
                .to_owned()
        }
    );

    Ok(())
}
