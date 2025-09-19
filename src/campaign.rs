use gtk::prelude::*;
use gtk::{glib, subclass::prelude::*};

use crate::config::Campaign;

use crate::config::SynchronizationOption;
use crate::errors::DragonDisplayError;

mod imp {

    use std::cell::{OnceCell, RefCell};

    use super::*;

    #[derive(Default)]
    pub struct DdCampaign {
        pub name: OnceCell<String>,
        pub path: OnceCell<String>,
        pub sync_option: RefCell<SynchronizationOption>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DdCampaign {
        const NAME: &'static str = "DdCampaign";
        type Type = super::DdCampaign;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for DdCampaign {}
}

glib::wrapper! {
    pub struct DdCampaign(ObjectSubclass<imp::DdCampaign>);
}

impl DdCampaign {
    pub fn new(name: String, path: String, sync_option: SynchronizationOption) -> Self {
        let obj = glib::Object::new::<Self>();
        obj.imp().name.set(name).unwrap();
        obj.imp().path.set(path).unwrap();
        obj.imp().sync_option.replace(sync_option);
        obj
    }

    pub fn from(campaign: Campaign) -> Self {
        let obj = glib::Object::new::<Self>();
        let imp = obj.imp();
        imp.name.set(campaign.name).unwrap();
        imp.path.set(campaign.path).unwrap();
        imp.sync_option.replace(campaign.sync_option);

        obj
    }

    /// Get a cloned version of the name
    pub fn name(&self) -> String {
        self.imp()
            .name
            .get()
            .expect("Campaign must have a name")
            .clone()
    }

    /// Get a cloned version of the path
    pub fn path(&self) -> String {
        self.imp()
            .path
            .get()
            .expect("Campaign must have a path")
            .clone()
    }

    /// Get a cloned version of the synchronization option
    pub fn sync_option(&self) -> SynchronizationOption {
        self.imp().sync_option.borrow().clone()
    }

    pub fn accesstokens(&self) -> Option<(String, String)> {
        let borrowed = self.imp().sync_option.borrow();
        if let SynchronizationOption::GoogleDrive {
            access_token,
            refresh_token,
            ..
        } = &*borrowed
        {
            return Some((access_token.clone(), refresh_token.clone()));
        }
        None
    }

    /// Sets the accesstoken and refreshtoken of the campaign if the campaign is a googledrive
    /// campaign, otherwise does nothing
    pub fn set_accesstoken(&self, accesstoken: String, refreshtoken: String) {
        let mut borrowed = self.imp().sync_option.borrow_mut();
        if let SynchronizationOption::GoogleDrive {
            access_token,
            refresh_token,
            ..
        } = &mut *borrowed
        {
            *access_token = accesstoken;
            *refresh_token = refreshtoken;
        }
    }
}
