use gtk::{glib, subclass::prelude::*};

use crate::config::Campaign;

use crate::config::SynchronizationOption;
use crate::setup::Token;

mod imp {

    use std::{
        cell::{OnceCell, RefCell},
        rc::Rc,
    };

    use super::*;

    #[derive(Default)]
    pub struct DdCampaign {
        pub name: Rc<OnceCell<String>>,
        pub path: Rc<OnceCell<String>>,
        pub sync_option: Rc<RefCell<SynchronizationOption>>,
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

    pub fn token(&self) -> Option<Token> {
        let borrowed = self.imp().sync_option.borrow();
        if let SynchronizationOption::GoogleDrive {
            access_token,
            refresh_token,
            ..
        } = &*borrowed
        {
            let token = Token {
                access_token: access_token.clone(),
                refresh_token: refresh_token.clone(),
            };
            return Some(token);
        }
        None
    }

    /// Get the synchronization folder id of the google drive campaign, if the campaign is
    pub fn sync_folder(&self) -> Option<String> {
        let binding = self.imp().sync_option.borrow();
        if let SynchronizationOption::GoogleDrive {
            google_drive_sync_folder,
            ..
        } = &*binding
        {
            return Some(google_drive_sync_folder.to_string());
        }
        None
    }

    /// Sets the accesstoken and refreshtoken of the campaign if the campaign is a googledrive
    /// campaign, otherwise does nothing
    pub fn set_token(&self, token: Token) {
        let mut borrowed = self.imp().sync_option.borrow_mut();
        if let SynchronizationOption::GoogleDrive {
            access_token,
            refresh_token,
            ..
        } = &mut *borrowed
        {
            *access_token = token.access_token;
            *refresh_token = token.refresh_token;
        }
    }

    /// Sets the google target folder id of the campaign if the campaign is a googledrive
    /// campaign, otherwise does nothing
    pub fn set_google_folder(&self, id: String) {
        let mut borrowed = self.imp().sync_option.borrow_mut();
        if let SynchronizationOption::GoogleDrive {
            google_drive_sync_folder,
            ..
        } = &mut *borrowed
        {
            *google_drive_sync_folder = id;
        }
    }
}

impl Default for DdCampaign {
    fn default() -> Self {
        glib::Object::new::<Self>()
    }
}
