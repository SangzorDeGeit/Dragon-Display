use std::cell::RefCell;
use gtk::{glib,subclass::prelude::{ObjectSubclass, ObjectSubclassExt, ObjectSubclassIsExt}};
use async_channel::Sender;

use crate::dragon_display::manage_campaign::config::Campaign;

use gtk::subclass::prelude::*;
use gtk::prelude::*;

mod imp {




    use super::*;
    // Object holding the campaign
    #[derive(Default)]
    pub struct RemoveButton{
        pub campaign: RefCell<Campaign>,
        // We make this an option so that the default trait is implemented
        // We should panic if the Option is None (the sender is not set)
        pub sender: RefCell<Option<Sender<Campaign>>>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for RemoveButton {
        const NAME: &'static str = "DragonDisplayRemoveButton";
        type Type = super::RemoveButton;
        type ParentType = gtk::Button;

    }

    // Trait shared by all GObjects
    impl ObjectImpl for RemoveButton {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for RemoveButton {}

    // Trait shared by all buttons
    impl ButtonImpl for RemoveButton {
        fn clicked(&self) {
                let c = self.campaign.borrow().to_owned();
                let _ = self.sender
                    .borrow()
                    .clone()
                    .unwrap()
                    .send_blocking(c).expect("Channel closed");
        }
    }
}


glib::wrapper! {
    pub struct RemoveButton(ObjectSubclass<imp::RemoveButton>)
        @extends gtk::Button, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl RemoveButton {
    pub fn new(campaign: Campaign, sender: Option<Sender<Campaign>>) -> Self {
        let object = glib::Object::new::<Self>();
        let imp = object.imp();
        object.set_label(&campaign.name);
        imp.campaign.replace(campaign);
        imp.sender.replace(sender);

        object
    }
}
