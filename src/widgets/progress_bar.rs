use async_channel::Receiver;
use gtk::glib;
use gtk::glib::spawn_future_local;
use gtk::subclass::prelude::ObjectSubclassIsExt;

pub enum ProgressMessage {
    Total { amount: usize },
    Current { amount: usize },
}

mod imp {
    use super::ProgressMessage;
    use std::cell::RefCell;

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
        pub update_receiver: RefCell<Option<Receiver<ProgressMessage>>>,
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
    pub fn new(receiver: Receiver<ProgressMessage>) -> Self {
        // set all properties
        let object = glib::Object::new::<Self>();
        let imp = object.imp();
        imp.update_receiver.replace(Some(receiver));

        Self::await_updates(&imp);
        object
    }

    /// this initialize function is called after the input variables for new() are set
    fn await_updates(imp: &imp::DdProgressBar) {
        let receiver = imp
            .update_receiver
            .borrow()
            .to_owned()
            .expect("No receiver found");
        let progress_bar = imp.progress_bar.clone();
        let mut total = 1.0;
        let mut current = 0.0;
        spawn_future_local(async move {
            while let Ok(update) = receiver.recv().await {
                match update {
                    ProgressMessage::Current { amount } => {
                        let new_current = current + amount as f64;
                        if new_current < total {
                            current = new_current;
                        }
                        let fraction = new_current / total;
                        progress_bar.set_fraction(fraction);
                        progress_bar.set_text(Some(&format!("{}/{}", new_current, total)));
                    }
                    ProgressMessage::Total { amount } => {
                        if amount > 0 {
                            total = amount as f64;
                        }
                    }
                }
            }
        });
    }
}
