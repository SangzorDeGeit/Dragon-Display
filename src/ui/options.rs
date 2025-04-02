use adw::Application;
use async_channel::Sender;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{gio, glib};
use gtk::{prelude::*, Adjustment};

use crate::APP_ID;
pub const MAX_COLUMN_ROW_AMOUNT: f64 = 20.0;
pub const MIN_COLUMN_ROW_AMOUNT: f64 = 0.0;

mod imp {
    use async_channel::Sender;
    use std::cell::RefCell;

    use glib::subclass::InitializingObject;
    use gtk::subclass::prelude::*;
    use gtk::{glib, Button, CompositeTemplate, SpinButton};
    use gtk::{prelude::*, template_callbacks};

    use crate::APP_ID;

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dragon/display/options.ui")]
    pub struct DdOptionsWindow {
        #[template_child]
        pub row: TemplateChild<SpinButton>,
        #[template_child]
        pub column: TemplateChild<SpinButton>,
        pub sender: RefCell<Option<Sender<()>>>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for DdOptionsWindow {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "DdOptionsWindow";
        type Type = super::DdOptionsWindow;
        type ParentType = gtk::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Button::ensure_type();

            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[template_callbacks]
    impl DdOptionsWindow {
        #[template_callback]
        fn handle_confirm(&self, _: Button) {
            let settings = gtk::gio::Settings::new(APP_ID);
            settings
                .set_int("imagegrid-row-amount", self.row.value() as i32)
                .expect("Could not update row");
            settings
                .set_int("imagegrid-column-amount", self.column.value() as i32)
                .expect("Could not update column");
            self.sender
                .borrow()
                .clone()
                .expect("No sender found")
                .send_blocking(())
                .expect("Channel closed");
            self.obj().clone().close();
        }

        #[template_callback]
        fn handle_default(&self, _: Button) {
            self.row.set_value(3.0);
            self.column.set_value(3.0);
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for DdOptionsWindow {
        fn constructed(&self) {
            // Call "constructed" on parent
            self.parent_constructed();
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for DdOptionsWindow {}

    // Trait shared by all windows
    impl WindowImpl for DdOptionsWindow {}

    // Trait shared by all application windows
    impl ApplicationWindowImpl for DdOptionsWindow {}
}

glib::wrapper! {
    pub struct DdOptionsWindow(ObjectSubclass<imp::DdOptionsWindow>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
            @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                        gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl DdOptionsWindow {
    pub fn new(app: &Application, sender: Sender<()>) -> Self {
        // set all properties
        let object = glib::Object::new::<Self>();
        object.set_property("application", app);
        let imp = object.imp();
        imp.row
            .set_range(MIN_COLUMN_ROW_AMOUNT, MAX_COLUMN_ROW_AMOUNT);
        imp.column
            .set_range(MIN_COLUMN_ROW_AMOUNT, MAX_COLUMN_ROW_AMOUNT);
        imp.sender.replace(Some(sender));
        let settings = gtk::gio::Settings::new(APP_ID);
        let columns = settings.int("imagegrid-column-amount") as f64;
        let rows = settings.int("imagegrid-row-amount") as f64;
        let row_adjustment = Adjustment::new(rows, 1.0, 20.0, 1.0, 10.0, 0.0);
        let column_adjustment = Adjustment::new(columns, 1.0, 20.0, 1.0, 10.0, 0.0);
        imp.row.set_adjustment(&row_adjustment);
        imp.column.set_adjustment(&column_adjustment);

        object
    }
}
