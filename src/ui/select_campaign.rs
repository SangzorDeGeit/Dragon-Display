use adw::Application;
use async_channel::Sender;
use gtk::prelude::ObjectExt;
use gtk::prelude::*;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{gio, glib};

use crate::dragon_display::setup::config::Campaign;
use crate::dragon_display::setup::SelectMessage;
use crate::widgets::campaign_button::CampaignButton;

mod imp {
    use std::cell::RefCell;

    use async_channel::Sender;
    use glib::subclass::InitializingObject;
    use gtk::prelude::StaticTypeExt;
    use gtk::subclass::prelude::*;
    use gtk::{glib, template_callbacks, Box, Button, CompositeTemplate, Grid, Label};

    use crate::dragon_display::setup::config::Campaign;
    use crate::dragon_display::setup::SelectMessage;

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dragon/display/select_campaign.ui")]
    pub struct SelectCampaignWindow {
        #[template_child]
        pub select_message: TemplateChild<Label>,
        #[template_child]
        pub campaign_grid: TemplateChild<Grid>,
        #[template_child]
        pub remove_add_box: TemplateChild<Box>,
        pub sender: RefCell<Option<Sender<SelectMessage>>>,
        pub campaign_list: RefCell<Vec<Campaign>>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for SelectCampaignWindow {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "DdSelectCampaignWindow";
        type Type = super::SelectCampaignWindow;
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
    impl SelectCampaignWindow {
        #[template_callback]
        fn handle_remove(&self, _: Button) {
            self.sender
                .borrow()
                .clone()
                .expect("No sender found")
                .send_blocking(SelectMessage::Remove)
                .expect("Channel closed");
        }

        #[template_callback]
        fn handle_add(&self, _: Button) {
            self.sender
                .borrow()
                .clone()
                .expect("No sender found")
                .send_blocking(SelectMessage::Add)
                .expect("Channel closed");
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for SelectCampaignWindow {
        fn constructed(&self) {
            // Call "constructed" on parent
            self.parent_constructed();
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for SelectCampaignWindow {}

    // Trait shared by all windows
    impl WindowImpl for SelectCampaignWindow {}

    // Trait shared by all application windows
    impl ApplicationWindowImpl for SelectCampaignWindow {}
}

glib::wrapper! {
    pub struct SelectCampaignWindow(ObjectSubclass<imp::SelectCampaignWindow>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
            @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                        gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl SelectCampaignWindow {
    pub fn new(
        app: &Application,
        sender: Option<Sender<SelectMessage>>,
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
    fn initialize(imp: &imp::SelectCampaignWindow) {
        let sender = imp.sender.borrow().clone().expect("No sender found");
        if imp.campaign_list.borrow().is_empty() {
            imp.select_message.set_text("You have no campaigns yet");
            // remove the 'remove button' from this window
            imp.remove_add_box.remove(
                &imp.remove_add_box
                    .first_child()
                    .expect("Could not find the remove button"),
            );
        } else {
            imp.select_message.set_text("Select a campaign");
        }

        let mut index = 0;
        for campaign in imp.campaign_list.borrow().iter() {
            let button = CampaignButton::new(campaign.clone(), Some(sender.clone()));
            imp.campaign_grid
                .attach(&button, index % 4, index / 4, 1, 1);
            index += 1;
        }
    }
}
