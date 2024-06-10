use adw::{Application, ApplicationWindow};
use gtk::glib::Properties;
use gtk::subclass::prelude::*;
use gtk::prelude::*;
use gtk::glib;
use std::cell::RefCell;
use crate::dragon_display::manage_campaign::config::Campaign;
use crate::dragon_display::start_dragon_display;

// Object holding the campaign
#[derive(Default)]
pub struct CampaignButton{
    campaign: Box<Campaign>,
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
    }
}