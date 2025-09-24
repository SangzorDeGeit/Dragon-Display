use adw::Application;
use gtk::prelude::{GtkWindowExt, ObjectExt};
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{gio, glib};

mod imp {
    use std::cell::OnceCell;

    use glib::subclass::InitializingObject;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::{glib, template_callbacks, Button, CompositeTemplate, Label};

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dragon/display/error_dialog.ui")]
    pub struct ErrorDialog {
        #[template_child]
        pub message_label: TemplateChild<Label>,
        pub fatal: OnceCell<bool>,
        pub app: OnceCell<adw::Application>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for ErrorDialog {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "DdErrorDialog";
        type Type = super::ErrorDialog;
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
    impl ErrorDialog {
        #[template_callback]
        fn handle_ok(&self, _: Button) {
            let fatal = self.fatal.get().expect("Expected fatal to be set");
            self.obj().destroy();
            if *fatal {
                self.app.get().expect("Expected app to be set").quit();
            }
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for ErrorDialog {
        fn constructed(&self) {
            // Call "constructed" on parent
            self.parent_constructed();
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for ErrorDialog {}

    // Trait shared by all windows
    impl WindowImpl for ErrorDialog {}

    // Trait shared by all application windows
    impl ApplicationWindowImpl for ErrorDialog {}
}

glib::wrapper! {
    pub struct ErrorDialog(ObjectSubclass<imp::ErrorDialog>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
            @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                        gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl ErrorDialog {
    /// Create a new modal error dialog
    pub fn new(app: &Application, msg: String, fatal: bool) -> Self {
        // set all properties
        let object = glib::Object::new::<Self>();
        object.set_modal(fatal);
        object
            .imp()
            .app
            .set(app.clone())
            .expect("Could not set app");
        object.imp().fatal.set(fatal).expect("Could not set fatal");

        let message: String;
        if fatal {
            message = format!("A fatal error occured:\n {}", msg);
        } else {
            message = format!("{}", msg);
        }
        object.imp().message_label.set_label(&message);

        object
    }
}
