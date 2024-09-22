use async_channel::Sender;
use gtk::{
    glib,
    subclass::prelude::{ObjectSubclass, ObjectSubclassIsExt},
};
use std::cell::RefCell;
use std::fs;
use std::io::{Error, ErrorKind};

use crate::dragon_display::setup::config::Campaign;
use crate::dragon_display::setup::SelectMessage;

use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {

    use super::*;
    // Object holding the campaign
    #[derive(Default)]
    pub struct CampaignButton {
        pub campaign: RefCell<Campaign>,
        // We make this an option so that the default trait is implemented
        // We should panic if the Option is None (the sender is not set)
        pub sender: RefCell<Option<Sender<SelectMessage>>>,
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
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for CampaignButton {}

    // Trait shared by all buttons
    impl ButtonImpl for CampaignButton {
        fn clicked(&self) {
            match fs::create_dir_all(&self.campaign.borrow().path) {
                Ok(_) => {
                    let c = self.campaign.borrow().to_owned();
                    self.sender
                        .borrow()
                        .clone()
                        .unwrap()
                        .send_blocking(SelectMessage::Campaign { campaign: c })
                        .expect("Channel closed");
                }
                Err(_) => {
                    let _ = self
                        .sender
                        .borrow()
                        .clone()
                        .unwrap()
                        .send_blocking(SelectMessage::Error {
                            error: Error::new(
                                ErrorKind::Other,
                                "An error occured while trying to create the folder for images",
                            ),
                            fatal: true,
                        })
                        .expect("Channel closed");
                }
            }
        }
    }
}

glib::wrapper! {
    pub struct CampaignButton(ObjectSubclass<imp::CampaignButton>)
        @extends gtk::Button, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl CampaignButton {
    pub fn new(campaign: Campaign, sender: Option<Sender<SelectMessage>>) -> Self {
        let object = glib::Object::new::<Self>();
        let imp = object.imp();
        object.set_label(&campaign.name);
        object.set_margin_end(6);
        object.set_margin_bottom(6);
        object.set_margin_start(6);
        object.set_margin_top(6);
        imp.campaign.replace(campaign);
        imp.sender.replace(sender);

        object
    }
}
