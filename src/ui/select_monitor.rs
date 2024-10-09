use std::io::{Error, ErrorKind};

use adw::Application;
use async_channel::Sender;
use gdk4::{Display, Monitor};
use gtk::prelude::*;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{gio, glib};

use crate::widgets::monitor_button::MonitorButton;

mod imp {
    use std::cell::RefCell;
    use std::io::Error;

    use async_channel::Sender;
    use gdk4::Monitor;
    use glib::subclass::InitializingObject;
    use gtk::subclass::prelude::*;
    use gtk::Grid;
    use gtk::{glib, CompositeTemplate};

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dragon/display/select_monitor.ui")]
    pub struct SelectMonitorWindow {
        #[template_child]
        pub monitor_grid: TemplateChild<Grid>,
        pub sender: RefCell<Option<Sender<Result<Monitor, Error>>>>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for SelectMonitorWindow {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "DdSelectMonitorWindow";
        type Type = super::SelectMonitorWindow;
        type ParentType = gtk::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for SelectMonitorWindow {
        fn constructed(&self) {
            // Call "constructed" on parent
            self.parent_constructed();
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for SelectMonitorWindow {}

    // Trait shared by all windows
    impl WindowImpl for SelectMonitorWindow {}

    // Trait shared by all application windows
    impl ApplicationWindowImpl for SelectMonitorWindow {}
}

glib::wrapper! {
    pub struct SelectMonitorWindow(ObjectSubclass<imp::SelectMonitorWindow>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
            @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                        gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl SelectMonitorWindow {
    pub fn new(app: &Application, sender: Sender<Result<Monitor, Error>>) -> Self {
        // set all properties
        let object = glib::Object::new::<Self>();
        object.set_property("application", app);
        let imp = object.imp();
        imp.sender.replace(Some(sender));
        Self::initialize(&imp);
        object
    }

    /// this initialize function is called after the input variables for new() are set
    fn initialize(imp: &imp::SelectMonitorWindow) {
        let sender = imp.sender.borrow().clone().expect("No sender found");
        let display = match Display::default() {
            Some(d) => d,
            None => {
                sender
                    .send_blocking(Err(Error::new(
                        ErrorKind::NotFound,
                        "Could not find a display",
                    )))
                    .expect("Channel closed");
                return;
            }
        };
        let mut i = 0;
        while let Some(monitor) = display.monitors().item(i) {
            let monitor = monitor
                .to_value()
                .get::<Monitor>()
                .expect("Value needs to be monitor");
            let monitor_button = MonitorButton::new(monitor, sender.clone());
            let index = i as i32;
            imp.monitor_grid
                .attach(&monitor_button, index % 4, index / 4, 1, 1);
            i += 1;
        }
    }
}
