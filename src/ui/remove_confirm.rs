use adw::Application;
use async_channel::Sender;
use gtk::prelude::ObjectExt;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{gio, glib};

use crate::config::Campaign;
use crate::setup_manager::AddRemoveMessage;

mod imp {
    use std::cell::RefCell;

    use async_channel::Sender;
    use glib::subclass::InitializingObject;
    use gtk::prelude::StaticTypeExt;
    use gtk::subclass::prelude::*;
    use gtk::{glib, template_callbacks, Button, CompositeTemplate, Label};

    use crate::config::Campaign;
    use crate::setup_manager::AddRemoveMessage;

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dragon/display/remove_confirm.ui")]
    pub struct RemoveConfirmWindow {
        #[template_child]
        pub message_label: TemplateChild<Label>,
        pub sender: RefCell<Option<Sender<AddRemoveMessage>>>,
        pub campaign: RefCell<Campaign>,
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
            self.sender
                .borrow()
                .clone()
                .expect("No sender found")
                .send_blocking(AddRemoveMessage::Campaign {
                    campaign: self.campaign.borrow().clone(),
                })
                .expect("Channel closed");
        }

        #[template_callback]
        fn handle_no(&self, _: Button) {
            self.sender
                .borrow()
                .clone()
                .expect("No sender found")
                .send_blocking(AddRemoveMessage::Cancel)
                .expect("Channel closed");
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for RemoveConfirmWindow {
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
    pub fn new(
        app: &Application,
        sender: Option<Sender<AddRemoveMessage>>,
        campaign: Campaign,
    ) -> Self {
        // set all properties
        let object = glib::Object::new::<Self>();
        let imp = object.imp();
        imp.campaign.replace(campaign);
        imp.sender.replace(sender);
        object.set_property("application", app);

        Self::initialize(&object.imp());
        object
    }

    /// this initialize function is called after the input variables for new() are set
    fn initialize(imp: &imp::RemoveConfirmWindow) {
        let message = format!(
            "Are you sure you want to delete {}?",
            imp.campaign.borrow().name
        );
        imp.message_label.set_text(&message);
    }
}
