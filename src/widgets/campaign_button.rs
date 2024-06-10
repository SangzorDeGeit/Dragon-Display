use adw::ApplicationWindow;
use gtk::{glib,subclass::prelude::{ObjectSubclass, ObjectSubclassExt, ObjectSubclassIsExt}};
use std::cell::RefCell;


use crate::dragon_display::manage_campaign::config::Campaign;

use gtk::subclass::prelude::*;
use gtk::prelude::*;
use crate::dragon_display::start_dragon_display;

mod imp {
    use std::fs;

    use crate::dragon_display::manage_campaign::gui::create_error_dialog;

    use super::*;
    // Object holding the campaign
    #[derive(Default)]
    pub struct CampaignButton{
        pub campaign: Box<Campaign>,
        pub window: RefCell<gtk::ApplicationWindow>,
        pub app: RefCell<adw::Application>
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for CampaignButton {
        const NAME: &'static str = "DragonDisplayCampaignButton";
        type Type = super::CampaignButton;
        type ParentType = gtk::Button;
    }

    // Trait shared by all GObjects
    impl ObjectImpl for CampaignButton {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().set_label(&self.campaign.name);
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for CampaignButton {}

    // Trait shared by all buttons
    impl ButtonImpl for CampaignButton {
        fn clicked(&self) {
            match fs::create_dir_all(self.campaign.path) {
                Ok(_) => (),
                Err(_) => {
                    create_error_dialog(&self.app, "Could not create image folder for the campaign!");
                    self.window.close();
                    return;
                }
            }
            self.window.take().close();
            start_dragon_display(&self.app, self.campaign)
        }
    }
}

glib::wrapper! {
    pub struct CampaignButton(ObjectSubclass<imp::CampaignButton>)
        @extends gtk::Button, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl CampaignButton {
    pub fn new(campaign: Campaign, app: RefCell<adw::Application>, window: RefCell<gtk::ApplicationWindow>) -> Self {
        let object = glib::Object::new::<Self>();
        let imp = object.imp();
        imp.campaign = Box::from(campaign);
        imp.app = app;
        imp.window = window;

        object
    }
}
