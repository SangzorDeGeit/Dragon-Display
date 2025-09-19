use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gdk4::Monitor;
use gtk::glib::clone;
use gtk::{glib, subclass::prelude::*};
use gtk::{prelude::*, Window};
use snafu::Report;

use crate::campaign::DdCampaign;
use crate::config::{
    read_campaign_from_config, remove_campaign_from_config, write_campaign_to_config, Campaign,
};
use crate::errors::DragonDisplayError;
use crate::gd_client::DragonDisplayGDClient;
use crate::ui::add_campaign::AddCampaignWindow;
use crate::ui::googledrive_connect::GoogledriveConnectWindow;
use crate::ui::googledrive_select_folder::DdGoogleFolderSelectWindow;
use crate::ui::remove_campaign::RemoveCampaignWindow;
use crate::ui::remove_confirm::RemoveConfirmWindow;
use crate::ui::select_campaign::SelectCampaignWindow;
use crate::widgets::progress_bar::DdProgressBar;
use crate::{runtime, try_emit};

#[derive(Clone, Debug)]
pub struct Token {
    pub access_token: String,
    pub refresh_token: String,
}

mod imp {

    use std::{cell::OnceCell, sync::OnceLock};

    use gdk4::Monitor;
    use gtk::glib::subclass::Signal;

    use crate::campaign::DdCampaign;

    use super::*;

    #[derive(Default)]
    pub struct DragonDisplaySetup {
        pub campaign: OnceCell<DdCampaign>,
        pub monitor: OnceCell<Monitor>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DragonDisplaySetup {
        const NAME: &'static str = "DdSetup";
        type Type = super::DragonDisplaySetup;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for DragonDisplaySetup {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder("finished")
                        .param_types([Monitor::static_type(), DdCampaign::static_type()])
                        .build(),
                    Signal::builder("error")
                        .param_types([String::static_type(), bool::static_type()])
                        .build(),
                ]
            })
        }
    }
}

glib::wrapper! {
    pub struct DragonDisplaySetup(ObjectSubclass<imp::DragonDisplaySetup>);
}

impl DragonDisplaySetup {
    pub fn new(app: &adw::Application) -> Result<Self, DragonDisplayError> {
        let obj = glib::Object::new::<Self>();
        obj.select_window(app);

        Ok(obj)
    }

    /// Create and present the select window
    pub fn select_window(&self, app: &adw::Application) {
        let campaign_list = try_emit!(self, read_campaign_from_config(), true);
        let window = SelectCampaignWindow::new(app, campaign_list);

        window.connect_remove_campaign(
            clone!(@weak self as obj, @weak window, @weak app => move |_| {
                window.destroy();
                obj.remove_window(&app);
            }),
        );

        window.connect_add_campaign(
            clone!(@weak self as obj, @weak window, @weak app => move |_| {
                window.destroy();
                obj.add_window(&app);
            }),
        );

        let imp = self.imp();
        window.connect_campaign(
            clone!(@weak self as obj, @weak imp, @weak window, @weak app => move |_, campaign| {
                window.destroy();
                imp.campaign.set(campaign);
                obj.monitor_window(&app);
            }),
        );

        window.present();
    }

    /// Create and present the remove window
    pub fn remove_window(&self, app: &adw::Application) {
        let campaign_list = try_emit!(self, read_campaign_from_config(), true);
        let window = RemoveCampaignWindow::new(app, campaign_list);

        window.connect_cancel(
            clone!(@weak self as obj, @weak window, @weak app => move |_| {
                window.destroy();
                obj.select_window(&app);
            }),
        );

        window.connect_remove(
            clone!(@weak self as obj, @weak window, @weak app => move |_, campaign| {
                window.destroy();
                obj.remove_confirm_window(&app, campaign);
            }),
        );

        window.present();
    }

    /// Create and present the remove confirmation window
    pub fn remove_confirm_window(&self, app: &adw::Application, campaign: DdCampaign) {
        let window = RemoveConfirmWindow::new(&app, &campaign);

        window.connect_no(
            clone!(@weak self as obj, @weak window, @weak app => move |_| {
                window.destroy();
                obj.remove_window(&app);
            }),
        );

        window.connect_yes(
            clone!(@weak self as obj, @weak window, @weak app, @weak campaign => move |_| {
                window.destroy();
                let campaign = Campaign::from(campaign);
                try_emit!(obj, remove_campaign_from_config(campaign, true), false);
                obj.remove_window(&app);
            }),
        );

        window.present();
    }

    /// Create and present the add campaign window
    pub fn add_window(&self, app: &adw::Application) {
        let window = AddCampaignWindow::new(app);

        window.connect_cancel(
            clone!(@weak self as obj, @weak window, @weak app => move |_| {
                window.destroy();
                obj.select_window(&app);
            }),
        );

        window.connect_error(clone!(@weak self as obj => move |_, msg, fatal| {
            obj.emit_by_name::<()>("error", &[&msg, &fatal]);
        }));

        window.connect_campaign_none(
            clone!(@weak self as obj, @weak window, @weak app => move |_, campaign| {
                window.destroy();
                let campaign = Campaign::from(campaign);
                try_emit!(obj, write_campaign_to_config(campaign), false);
                obj.select_window(&app);
            }),
        );

        window.connect_campaign_gd(
            clone!(@weak self as obj, @weak window, @weak app => move |_, campaign| {
                window.destroy();
                obj.googledrive_connect(&app, campaign, false);
            }),
        );

        window.present();
    }

    /// Create and present the google drive connect window
    pub fn googledrive_connect(
        &self,
        app: &adw::Application,
        campaign: DdCampaign,
        reconnect: bool,
    ) {
        let client = DragonDisplayGDClient::new();
        let window = GoogledriveConnectWindow::new(&app, reconnect);

        window.connect_connect(clone!(@weak client => move |_| {
            runtime().spawn(async move {
                client.connect().await;
            });
        }));

        window.connect_cancel(
            clone!(@weak self as obj, @weak client, @weak window, @weak app => move |_| {
                window.destroy();
                try_emit!(obj, client.shutdown_server(), false);
                obj.select_window(&app);
            }),
        );

        client.connect_url(clone!(@weak window => move |_, url| {
            window.update_url(&url);
        }));

        client.connect_accesstoken(
            clone!(@weak self as obj, @weak campaign, @weak window, @weak app => move |_, accesstoken, refreshtoken| {
                window.destroy();
                campaign.set_accesstoken(accesstoken, refreshtoken);
                obj.googledrive_selectfolder(&app, campaign);
            }),
        );

        client.connect_error(clone!(@weak self as obj => move |_, msg, fatal| {
            obj.emit_by_name::<()>("error", &[&msg, &fatal])
        }));

        window.present();
    }

    /// Create and present the google drive load folders, a progress bar for loading all folders in
    /// the target drive
    pub fn googledrive_loadfolders(&self, app: &adw::Application, campaign: DdCampaign) {
        let progbar = DdProgressBar::new();
        let window = Window::builder().application(app).child(&progbar).build();
        let client = DragonDisplayGDClient::new();
        let tokens = campaign.accesstokens().expect("Expected tokens");
        let token = Rc::new(RefCell::new(Token {
            access_token: tokens.0,
            refresh_token: tokens.1,
        }));

        let token_snapshot = token.borrow().clone();
        runtime().spawn(clone!(@weak client => async move {
            client
                .total_folders(token_snapshot)
                .await;
        }));

        client.connect_total_folders(
            clone!(@weak progbar, @weak token, @weak client => move |_, amount| {
                progbar.update_total(amount as usize);
                let token_snapshot = token.borrow().clone();
                runtime().spawn(async move {
                    client.list_folders(token_snapshot);
                });
            }),
        );

        client.connect_reconnect(
            clone!(@weak self as obj, @weak window, @weak app, @weak campaign => move |_| {
                window.destroy();
                obj.googledrive_connect(&app, campaign, true);
            }),
        );

        client.connect_refresh_total(clone!(@weak token, @weak client => move |_| {
            let token_snapshot = token.borrow().clone();
            runtime().spawn(async move {
                client.refresh_client(token_snapshot).await;
            });
        }));

        window.present();
    }

    /// Create and present the google drive select folder window
    pub fn googledrive_selectfolder(&self, app: &adw::Application, campaign: DdCampaign) {
        let window = DdGoogleFolderSelectWindow::new(app);
        let client = DragonDisplayGDClient::new();
        let progbar = DdProgressBar::new();
        window.set_progress_bar(&progbar);
        let consecutive_refresh_calls: Cell<u8> = Cell::new(0);

        window.present();
    }

    pub fn monitor_window(&self, app: &adw::Application) {}

    /// Signal emitted when a monitor is selected, sends the selected campaign and monitor
    pub fn connect_finished<F: Fn(&Self, Monitor, DdCampaign) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            "finished",
            true,
            glib::closure_local!(|window, monitor, campaign| {
                f(window, monitor, campaign);
            }),
        )
    }

    /// Signal emitted when a monitor is selected, sends the selected campaign and monitor
    pub fn connect_error<F: Fn(&Self, String, bool) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            "error",
            true,
            glib::closure_local!(|window, msg, fatal| {
                f(window, msg, fatal);
            }),
        )
    }

    /// Emit an error message based on the input error
    pub fn emit_error(&self, err: DragonDisplayError, fatal: bool) {
        let msg = Report::from_error(err).to_string();
        self.emit_by_name::<()>("error", &[&msg, &fatal]);
    }
}
