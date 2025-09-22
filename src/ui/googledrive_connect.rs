use adw::Application;
use google_drive::AccessToken;
use gtk::prelude::ObjectExt;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{gio, glib};

pub enum InitializeMessage {
    UserConsentUrl { url: String },
    Token { token: AccessToken },
    Error { error: anyhow::Error },
}

mod imp {
    use gtk::glib::subclass::Signal;
    use std::sync::OnceLock;

    use glib::subclass::InitializingObject;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::{glib, template_callbacks, Button, CompositeTemplate, Label};

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dragon/display/googledrive_connect.ui")]
    pub struct GoogledriveConnectWindow {
        #[template_child]
        pub message_label: TemplateChild<Label>,
        #[template_child]
        pub link_label: TemplateChild<Label>,
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
            self.obj().emit_by_name::<()>("cancel", &[]);
        }

        #[template_callback]
        fn handle_connect(&self, button: Button) {
            self.obj().emit_by_name::<()>("connect", &[]);
            button.set_sensitive(false);
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for GoogledriveConnectWindow {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder("cancel").build(),
                    Signal::builder("connect").build(),
                ]
            })
        }
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
    pub fn new(app: &Application, reconnect: bool) -> Self {
        // set all properties
        let object = glib::Object::new::<Self>();
        object.set_property("application", app);
        let imp = object.imp();

        let message: &str;
        if reconnect {
            message = "Google Drive session is expired, please reconnect to continue using google drive synchronization.";
        } else {
            message = "In order to use Google Drive synchronization you need to give Dragon-Display permission to connect to your Google Account";
        }
        imp.message_label.set_text(message);

        object
    }

    /// Update the label of connect message to add a url
    pub fn update_url(&self, url: &str) {
        let msg = format!("If the browser does not open automatically, copy paste the following link into your browser: {}" , url);
        self.imp().link_label.set_text(&msg);
    }

    /// Signal emitted when the cancel button is pressed
    pub fn connect_cancel<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "cancel",
            true,
            glib::closure_local!(|window| {
                f(window);
            }),
        )
    }

    /// Signal emitted when the connect button is pressed
    pub fn connect_connect<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "connect",
            true,
            glib::closure_local!(|window| {
                f(window);
            }),
        )
    }
}
