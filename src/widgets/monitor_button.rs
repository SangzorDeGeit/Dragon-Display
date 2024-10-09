use async_channel::Sender;
use gdk4::Monitor;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::RefCell;
use std::io::Error;

mod imp {

    use gdk4::Monitor;
    use std::io::Error;

    use super::*;
    // Object holding the campaign
    #[derive(Default)]
    pub struct MonitorButton {
        pub monitor: RefCell<Option<Monitor>>,
        // We make this an option so that the default trait is implemented
        // We should panic if the Option is None (the sender is not set)
        pub sender: RefCell<Option<Sender<Result<Monitor, Error>>>>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for MonitorButton {
        const NAME: &'static str = "DragonDisplayMonitorButton";
        type Type = super::MonitorButton;
        type ParentType = gtk::Button;
    }

    // Trait shared by all GObjects
    impl ObjectImpl for MonitorButton {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for MonitorButton {}

    // Trait shared by all buttons
    impl ButtonImpl for MonitorButton {
        fn clicked(&self) {
            let monitor = self.monitor.borrow().clone().expect("No monitor found");
            self.sender
                .borrow()
                .clone()
                .expect("No sender found")
                .send_blocking(Ok(monitor))
                .expect("Channel closed");
        }
    }
}

glib::wrapper! {
    pub struct MonitorButton(ObjectSubclass<imp::MonitorButton>)
        @extends gtk::Button, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl MonitorButton {
    pub fn new(monitor: Monitor, sender: Sender<Result<Monitor, Error>>) -> Self {
        let object = glib::Object::new::<Self>();
        let imp = object.imp();
        imp.monitor.replace(Some(monitor.clone()));
        imp.sender.replace(Some(sender));
        let label = format!(
            "{}cm x {}cm",
            monitor.height_mm() / 10,
            monitor.width_mm() / 10
        );
        object.set_label(&label);
        object
    }
}
