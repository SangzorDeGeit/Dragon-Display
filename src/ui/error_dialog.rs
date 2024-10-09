use std::io::Error;

use adw::Application;
use gtk::prelude::ObjectExt;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{gio, glib};

mod imp {
    use std::cell::{Cell, RefCell};

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
        pub error_msg: RefCell<String>,
        pub fatal: Cell<bool>,
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
        fn handle_ok(&self, button: Button) {
            let binding = button.root().expect("No root found");
            let window = binding
                .downcast_ref::<gtk::Window>()
                .expect("Root was not a window");
            let app = self.obj().application().expect("No application found");
            window.close();
            if self.fatal.get() {
                app.quit();
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
    pub fn new(app: &Application, error: Error, fatal: bool) -> Self {
        // set all properties
        let object = glib::Object::new::<Self>();
        object.set_property("application", app);
        let imp = object.imp();
        imp.error_msg.replace(error.to_string());
        imp.fatal.replace(fatal);

        Self::initialize(&imp);
        object
    }

    /// this initialize function is called after the input variables for new() are set
    fn initialize(imp: &imp::ErrorDialog) {
        let message: String;
        if imp.fatal.get() {
            message = format!("A fatal error occured:\n {}", imp.error_msg.borrow());
        } else {
            message = format!("{}", imp.error_msg.borrow());
        }
        imp.message_label.set_text(&message);
    }
}
