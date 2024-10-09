use adw::Application;
use async_channel::Sender;
use gtk::prelude::ObjectExt;
use gtk::prelude::*;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{gio, glib};

use crate::dragon_display::setup::config::Campaign;
use crate::dragon_display::setup::AddRemoveMessage;
use crate::widgets::remove_button::RemoveButton;

mod imp {
    use std::cell::RefCell;

    use async_channel::Sender;
    use glib::subclass::InitializingObject;
    use gtk::prelude::StaticTypeExt;
    use gtk::subclass::prelude::*;
    use gtk::{glib, template_callbacks, Button, CompositeTemplate, Grid};

    use crate::dragon_display::setup::config::Campaign;
    use crate::dragon_display::setup::AddRemoveMessage;

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dragon/display/remove_campaign.ui")]
    pub struct RemoveCampaignWindow {
        #[template_child]
        pub campaign_grid: TemplateChild<Grid>,
        pub sender: RefCell<Option<Sender<AddRemoveMessage>>>,
        pub campaign_list: RefCell<Vec<Campaign>>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for RemoveCampaignWindow {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "DdRemoveCampaignWindow";
        type Type = super::RemoveCampaignWindow;
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
    impl RemoveCampaignWindow {
        #[template_callback]
        fn handle_cancel(&self, _: Button) {
            self.sender
                .borrow()
                .clone()
                .expect("No sender found")
                .send_blocking(AddRemoveMessage::Cancel)
                .expect("Channel closed");
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for RemoveCampaignWindow {
        fn constructed(&self) {
            // Call "constructed" on parent
            self.parent_constructed();
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for RemoveCampaignWindow {}

    // Trait shared by all windows
    impl WindowImpl for RemoveCampaignWindow {}

    // Trait shared by all application windows
    impl ApplicationWindowImpl for RemoveCampaignWindow {}
}

glib::wrapper! {
    pub struct RemoveCampaignWindow(ObjectSubclass<imp::RemoveCampaignWindow>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
            @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                        gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl RemoveCampaignWindow {
    pub fn new(
        app: &Application,
        sender: Option<Sender<AddRemoveMessage>>,
        campaign_list: Vec<Campaign>,
    ) -> Self {
        // set all properties
        let object = glib::Object::new::<Self>();
        let imp = object.imp();
        imp.campaign_list.replace(campaign_list);
        imp.sender.replace(sender);
        object.set_property("application", app);

        Self::initialize(&object.imp());
        object
    }

    /// this initialize function is called after the input variables for new() are set
    fn initialize(imp: &imp::RemoveCampaignWindow) {
        let mut index = 0;
        for campaign in imp.campaign_list.borrow().iter() {
            let button = RemoveButton::new(campaign.clone(), imp.sender.borrow().clone());
            imp.campaign_grid
                .attach(&button, index % 4, index / 4, 1, 1);
            index += 1;
        }
    }
}
