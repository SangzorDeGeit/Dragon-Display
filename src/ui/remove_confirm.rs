use gtk::prelude::ObjectExt;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{gio, glib};

use crate::campaign::DdCampaign;

mod imp {
    use std::sync::OnceLock;

    use glib::subclass::InitializingObject;
    use gtk::glib::object::ObjectExt;
    use gtk::glib::subclass::Signal;
    use gtk::prelude::StaticTypeExt;
    use gtk::subclass::prelude::*;
    use gtk::{glib, template_callbacks, Button, CompositeTemplate, Label};

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dragon/display/remove_confirm.ui")]
    pub struct RemoveConfirmWindow {
        #[template_child]
        pub message_label: TemplateChild<Label>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for RemoveConfirmWindow {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "DdRemoveConfirmWindow";
        type Type = super::RemoveConfirmWindow;
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
    impl RemoveConfirmWindow {
        #[template_callback]
        fn handle_yes(&self, _: Button) {
            self.obj().emit_by_name::<()>("yes", &[]);
        }

        #[template_callback]
        fn handle_no(&self, _: Button) {
            self.obj().emit_by_name::<()>("no", &[]);
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for RemoveConfirmWindow {
        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder("no").build(),
                    Signal::builder("yes").build(),
                ]
            })
        }
        fn constructed(&self) {
            // Call "constructed" on parent
            self.parent_constructed();
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for RemoveConfirmWindow {}

    // Trait shared by all windows
    impl WindowImpl for RemoveConfirmWindow {}

    // Trait shared by all application windows
    impl ApplicationWindowImpl for RemoveConfirmWindow {}
}

glib::wrapper! {
    pub struct RemoveConfirmWindow(ObjectSubclass<imp::RemoveConfirmWindow>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
            @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                        gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl RemoveConfirmWindow {
    pub fn new(app: &gtk::Application, campaign: &DdCampaign) -> Self {
        let object = glib::Object::new::<Self>();
        object.set_property("application", app);

        let message = format!("Are you sure you want to delete {}?", campaign.name());
        object.imp().message_label.set_text(&message);

        object
    }

    /// The signal emitted when the yes button is clicked
    pub fn connect_yes<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "yes",
            true,
            glib::closure_local!(|window| {
                f(window);
            }),
        )
    }

    /// The signal emitted when the no button is clicked
    pub fn connect_no<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "no",
            true,
            glib::closure_local!(|window| {
                f(window);
            }),
        )
    }
}
