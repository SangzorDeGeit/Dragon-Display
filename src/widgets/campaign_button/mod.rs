pub mod imp;

use glib::Object;
use gtk::glib;

use crate::dragon_display::manage_campaign::config::Campaign;


glib::wrapper! {
    pub struct CampaignButton(ObjectSubclass<imp::CampaignButton>)
        @extends gtk::Button, gtk::Widget,
        @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl CampaignButton {
    pub fn new() -> Self {
        Object::builder().build()
    }

    pub fn with_label(label: &str) -> Self{
        Object::builder().property("label", label).build()
    }

    pub fn set_campaign(campaign: Box<Campaign>) {
        
    }

}

impl Default for CampaignButton {
    fn default() -> Self {
        Self::new()
    }
}