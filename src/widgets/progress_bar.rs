use gtk::glib;
use gtk::subclass::prelude::ObjectSubclassIsExt;

pub enum ProgressMessage {
    Total { amount: usize },
    Current { amount: usize },
}

mod imp {
    use super::ProgressMessage;
    use std::cell::{Cell, RefCell};

    use async_channel::Receiver;
    use glib::subclass::InitializingObject;
    use gtk::subclass::prelude::*;
    use gtk::{glib, CompositeTemplate};
    use gtk::{prelude::*, ProgressBar};

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dragon/display/progress_bar.ui")]
    pub struct DdProgressBar {
        #[template_child]
        pub progress_bar: TemplateChild<ProgressBar>,
        pub current: Cell<usize>,
        pub total: Cell<usize>,
        pub operation: RefCell<String>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for DdProgressBar {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "DdProgressBar";
        type ParentType = gtk::Widget;
        type Type = super::DdProgressBar;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_layout_manager_type::<gtk::BoxLayout>();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for DdProgressBar {
        fn constructed(&self) {
            // Call "constructed" on parent
            self.parent_constructed();
        }

        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for DdProgressBar {}
}

glib::wrapper! {
    pub struct DdProgressBar(ObjectSubclass<imp::DdProgressBar>)
        @extends gtk::Widget,
            @implements gtk::Actionable, gtk::Accessible, gtk::Buildable,
                        gtk::ConstraintTarget;
}

impl DdProgressBar {
    pub fn new(operation: String) -> Self {
        // set all properties
        let object = glib::Object::new::<Self>();
        object.imp().total.set(1);
        object.imp().current.set(0);
        object.imp().operation.replace(operation);
        object
    }

    pub fn set_operation(&self, operation: String) {
        self.imp().operation.replace(operation);
    }

    /// Update the total, will do nothing if the output is not at least 1 or the total current
    /// amount
    pub fn update_total(&self, total: usize) {
        let current = self.imp().current.get();
        if total <= 0 || total <= current {
            return;
        }
        self.imp().total.set(total);

        let text = format!("{}: {}/{}", self.imp().operation.borrow(), current, total);
        let fraction = current as f64 / total as f64;

        self.imp().progress_bar.set_text(Some(&text));
        self.imp().progress_bar.set_fraction(fraction);
    }

    /// Update the current progress adding the amount to the current value, this function will not
    /// update if the new total current is higher then the total
    pub fn update_progress(&self, amount: usize) {
        let total = self.imp().total.get();
        let current = self.imp().current.get() + amount;
        if current >= total {
            return;
        }
        self.imp().current.set(current);

        let text = format!("{}/{}", current, self.imp().total.get());
        let fraction = self.imp().current.get() as f64 / self.imp().total.get() as f64;

        self.imp().progress_bar.set_text(Some(&text));
        self.imp().progress_bar.set_fraction(fraction);
    }
}
