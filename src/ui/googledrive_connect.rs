use adw::Application;
use async_channel::Sender;
use google_drive::AccessToken;
use gtk::prelude::ObjectExt;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{gio, glib};

use crate::config::Campaign;
use crate::setup_manager::AddRemoveMessage;

pub enum InitializeMessage {
    UserConsentUrl { url: String },
    Token { token: AccessToken },
    Error { error: anyhow::Error },
}

mod imp {
    use super::InitializeMessage;
    use async_channel::Sender;
    use gtk::glib::spawn_future_local;
    use std::cell::RefCell;

    use glib::subclass::InitializingObject;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::{glib, template_callbacks, Button, CompositeTemplate, Label};

    use crate::google_drive::initialize_client;
    use crate::config::{Campaign, SynchronizationOption};
    use crate::setup_manager::AddRemoveMessage;
    use crate::runtime;

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dragon/display/googledrive_connect.ui")]
    pub struct GoogledriveConnectWindow {
        #[template_child]
        pub message_label: TemplateChild<Label>,
        #[template_child]
        pub link_label: TemplateChild<Label>,
        pub campaign_sender: RefCell<Option<Sender<AddRemoveMessage>>>,
        pub shutdown_sender: RefCell<Option<Sender<()>>>,
        pub campaign: RefCell<Campaign>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for GoogledriveConnectWindow {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "DdGoogledriveConnectWindow";
        type Type = super::GoogledriveConnectWindow;
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
    impl GoogledriveConnectWindow {
        #[template_callback]
        fn handle_cancel(&self, _: Button) {
            if let Some(shutdown_sender) = self.shutdown_sender.borrow().clone() {
                shutdown_sender.send_blocking(()).expect("Channel closed");
            }
            self.campaign_sender
                .borrow()
                .clone()
                .expect("No campaign sender found")
                .send_blocking(AddRemoveMessage::Cancel)
                .expect("Channel closed");
        }

        #[template_callback]
        fn handle_connect(&self, button: Button) {
            let (shutdown_sender, shutdown_receiver) = async_channel::bounded(1);
            let (backend_sender, backend_receiver) = async_channel::unbounded();
            runtime().spawn(async move {
                initialize_client(backend_sender, shutdown_receiver).await;
            });
            button.set_sensitive(false);
            self.shutdown_sender.replace(Some(shutdown_sender));

            let link_label = self.link_label.clone();
            let campaign = self.campaign.borrow().clone();
            let name = campaign.name;
            let path = campaign.path;
            let google_folder = match campaign.sync_option {
                SynchronizationOption::None => {
                    panic!("Connect window was called with None sync option")
                }
                SynchronizationOption::GoogleDrive {
                    google_drive_sync_folder: f,
                    ..
                } => f,
            };
            let campaign_sender = self
                .campaign_sender
                .borrow()
                .clone()
                .expect("No sender found");
            spawn_future_local(async move {
                while let Ok(message) = backend_receiver.recv().await {
                    match message {
                        InitializeMessage::UserConsentUrl { url } => link_label.set_text(&format!(
                                "If the browser does not open automatically, copy paste the following link into your browser: {}"
                                , &url)),
                        InitializeMessage::Token { token } => {
                            let new_campaign = Campaign::new_googledrive(
                                name.clone(), 
                                path.clone(), 
                                token.access_token, 
                                token.refresh_token, 
                                google_folder.clone()
                            ); 
                            campaign_sender.send_blocking(AddRemoveMessage::Campaign { campaign: new_campaign }).expect("Channel closed");
                        }
                        InitializeMessage::Error { error } => campaign_sender
                            .send_blocking(AddRemoveMessage::Error {
                                error, 
                                fatal: false })
                            .expect("Channel closed"),
                    }
                }
            });
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for GoogledriveConnectWindow {
        fn constructed(&self) {
            // Call "constructed" on parent
            self.parent_constructed();
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for GoogledriveConnectWindow {}

    // Trait shared by all windows
    impl WindowImpl for GoogledriveConnectWindow {}

    // Trait shared by all application windows
    impl ApplicationWindowImpl for GoogledriveConnectWindow {}
}

glib::wrapper! {
    pub struct GoogledriveConnectWindow(ObjectSubclass<imp::GoogledriveConnectWindow>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
            @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                        gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl GoogledriveConnectWindow {
    pub fn new(
        app: &Application,
        campaign: Campaign,
        campaign_sender: Sender<AddRemoveMessage>,
        reconnect: bool,
    ) -> Self {
        // set all properties
        let object = glib::Object::new::<Self>();
        object.set_property("application", app);
        let imp = object.imp();
        imp.campaign.replace(campaign);
        imp.campaign_sender.replace(Some(campaign_sender));

        let message: &str;
        if reconnect {
            message = "Google Drive session is expired, please reconnect to continue using google drive synchronization.";
        } else {
            message = "In order to use Google Drive synchronization you need to give Dragon-Display permission to connect to your Google Account";
        }
        imp.message_label.set_text(message);

        object
    }
}
