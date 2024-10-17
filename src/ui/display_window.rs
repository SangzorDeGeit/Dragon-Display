use adw::Application;
use gtk::prelude::ObjectExt;
use gtk::{gio, glib};

mod imp {
    use async_channel::Receiver;
    use std::cell::RefCell;

    use glib::subclass::InitializingObject;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::{glib, Box, Button, CompositeTemplate};

    use crate::ui::control_window::UpdateDisplayMessage;

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dragon/display/display_window.ui")]
    pub struct DdDisplayWindow {
        #[template_child]
        pub content: TemplateChild<Box>,
        pub receiver: RefCell<Option<Receiver<UpdateDisplayMessage>>>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for DdDisplayWindow {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "DdDdDisplayWindow";
        type Type = super::DdDisplayWindow;
        type ParentType = gtk::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Button::ensure_type();

            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for DdDisplayWindow {
        fn constructed(&self) {
            // Call "constructed" on parent
            self.parent_constructed();
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for DdDisplayWindow {}

    // Trait shared by all windows
    impl WindowImpl for DdDisplayWindow {}

    // Trait shared by all application windows
    impl ApplicationWindowImpl for DdDisplayWindow {}
}

glib::wrapper! {
    pub struct DdDisplayWindow(ObjectSubclass<imp::DdDisplayWindow>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
            @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                        gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl DdDisplayWindow {
    pub fn new(app: &Application) -> Self {
        // set all properties
        let object = glib::Object::new::<Self>();
        object.set_property("application", app);
        object
    }
}
