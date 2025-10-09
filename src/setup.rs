use std::cell::RefCell;
use std::rc::Rc;

use gtk::glib::clone;
use gtk::{glib, subclass::prelude::*};
use gtk::{prelude::*, Window};
use snafu::{Report, ResultExt};

use crate::config::{
    read_campaign_from_config, remove_campaign_from_config, write_campaign_to_config, Campaign,
    SynchronizationOption,
};
use crate::errors::{DragonDisplayError, SendBackendSnafu};
use crate::gd_client::{DragonDisplayGDClient, GdClientEvent};
use crate::program::DragonDisplayProgram;
use crate::ui::add_campaign::AddCampaignWindow;
use crate::ui::googledrive_connect::GoogledriveConnectWindow;
use crate::ui::googledrive_select_folder::DdGoogleFolderSelectWindow;
use crate::ui::remove_campaign::RemoveCampaignWindow;
use crate::ui::remove_confirm::RemoveConfirmWindow;
use crate::ui::select_campaign::SelectCampaignWindow;
use crate::ui::select_monitor::SelectMonitorWindow;
use crate::widgets::google_folder_object::GoogleFolderObject;
use crate::widgets::progress_bar::DdProgressBar;
use crate::{runtime, try_emit};

#[derive(Clone, Debug)]
pub struct Token {
    pub access_token: String,
    pub refresh_token: String,
}

/// The state of the Gdclient if it exists
#[derive(Clone, Debug)]
pub enum GdClientState {
    /// State for general info acquisition
    General,
    /// State for folder indexing to generate a foldertree
    ListFolders {
        folders: Vec<GoogleFolderObject>,
        indexed_folders: Vec<GoogleFolderObject>,
    },
    /// State for downloading files
    DownloadFiles {
        total_needed: usize,
        to_download: Vec<(String, String)>,
    },
}

pub enum GDConnectCallingFn {
    Add,
    LoadFolders,
    SyncFiles,
}

impl GdClientState {
    /// Tries to pop an element from the folders variable of gdClientState, can silently fail
    fn pop_folder(&mut self) {
        match self {
            Self::ListFolders { folders, .. } => drop(folders.pop()),
            Self::DownloadFiles { to_download, .. } => drop(to_download.pop()),
            Self::General => (),
        }
    }

    /// Add an indexed folder to the gdclientstate, an indexed folder is a folder that has the
    /// children variable set.
    fn new_indexed(&mut self, folder: GoogleFolderObject) {
        match self {
            Self::ListFolders {
                indexed_folders, ..
            } => {
                indexed_folders.push(folder);
            }
            _ => (),
        }
    }

    /// Returns the indexed folders object in the client state, panics if the state is TotalFolders
    fn indexed_folders(&self) -> Vec<GoogleFolderObject> {
        match self {
            Self::ListFolders {
                indexed_folders, ..
            } => indexed_folders.clone(),
            _ => panic!("function called while not listing folders"),
        }
    }
}

impl Default for GdClientState {
    fn default() -> Self {
        Self::General
    }
}

mod imp {

    use std::{
        cell::{Cell, OnceCell},
        sync::OnceLock,
    };

    use gdk4::Monitor;
    use gtk::glib::subclass::Signal;

    use crate::campaign::DdCampaign;

    use super::*;

    #[derive(Default)]
    pub struct DragonDisplaySetup {
        pub campaign: RefCell<DdCampaign>,
        pub monitor: OnceCell<Monitor>,
        /// a state variable indicating whether the program is in setup mode
        pub setup: Cell<bool>,
        /// a state variable for remembering between refresh operations of a gdClient
        pub gd_client_state: RefCell<GdClientState>,
        /// The main dragon display program is set when the setup is complete
        pub program: OnceCell<DragonDisplayProgram>,
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
                vec![Signal::builder("error")
                    .param_types([String::static_type(), bool::static_type()])
                    .build()]
            })
        }
    }
}

glib::wrapper! {
    pub struct DragonDisplaySetup(ObjectSubclass<imp::DragonDisplaySetup>);
}

impl DragonDisplaySetup {
    pub fn new() -> Self {
        let obj = glib::Object::new::<Self>();
        obj.imp().setup.set(true);
        obj
    }

    /// Create and present the select window
    pub fn select_window(&self, app: &adw::Application) {
        let campaign_list = try_emit!(self, read_campaign_from_config(), true);
        let window = SelectCampaignWindow::new(app, campaign_list);

        window.connect_remove_campaign(clone!(@weak self as obj, @weak app => move |window| {
            window.destroy();
            obj.remove_window(&app);
        }));

        window.connect_add_campaign(clone!(@weak self as obj, @weak app => move |window| {
            window.destroy();
            obj.add_window(&app);
        }));

        let imp = self.imp();
        window.connect_campaign(
            clone!(@weak self as obj, @weak imp, @weak app => move |window, campaign| {
                window.destroy();
                obj.imp().campaign.replace(campaign);
                match imp.campaign.borrow().sync_option() {
                    SynchronizationOption::None => obj.monitor_window(&app),
                    SynchronizationOption::GoogleDrive { .. } => obj.googledrive_synchronize(&app),
                };
            }),
        );

        window.present();
    }

    /// Create and present the remove window
    pub fn remove_window(&self, app: &adw::Application) {
        let campaign_list = try_emit!(self, read_campaign_from_config(), true);
        if campaign_list.is_empty() {
            self.select_window(app);
            return;
        }
        let window = RemoveCampaignWindow::new(app, campaign_list);

        window.connect_cancel(clone!(@weak self as obj, @weak app => move |window| {
            window.destroy();
            obj.select_window(&app);
        }));

        window.connect_remove(
            clone!(@weak self as obj, @weak app => move |window, campaign| {
                window.destroy();
                obj.imp().campaign.replace(campaign);
                obj.remove_confirm_window(&app);
            }),
        );

        window.present();
    }

    /// Create and present the remove confirmation window
    pub fn remove_confirm_window(&self, app: &adw::Application) {
        let window = RemoveConfirmWindow::new(&app, &self.imp().campaign.borrow());

        window.connect_no(clone!(@weak self as obj, @weak app => move |window| {
            window.destroy();
            obj.remove_window(&app);
        }));

        window.connect_yes(clone!(@weak self as obj, @weak app => move |window| {
            let campaign = Campaign::from(&obj.imp().campaign.borrow());
            try_emit!(obj, remove_campaign_from_config(campaign, true), false);
            window.destroy();
            obj.remove_window(&app);
        }));

        window.present();
    }

    /// Create and present the add campaign window
    pub fn add_window(&self, app: &adw::Application) {
        let window = AddCampaignWindow::new(app);

        window.connect_cancel(clone!(@weak self as obj, @weak app => move |window| {
            window.destroy();
            obj.select_window(&app);
        }));

        window.connect_error(clone!(@weak self as obj => move |_, msg, fatal| {
            obj.emit_by_name::<()>("error", &[&msg, &fatal]);
        }));

        window.connect_campaign_none(
            clone!(@weak self as obj, @weak app => move |window, campaign| {
                window.destroy();
                let campaign = Campaign::from(&campaign);
                try_emit!(obj, write_campaign_to_config(campaign), false);
                obj.select_window(&app);
            }),
        );

        window.connect_campaign_gd(
            clone!(@weak self as obj, @weak app => move |window, campaign| {
                window.destroy();
                obj.imp().campaign.replace(campaign);
                obj.googledrive_connect(&app, GDConnectCallingFn::Add);
            }),
        );

        window.present();
    }

    /// Create and present the google drive connect window
    pub fn googledrive_connect(&self, app: &adw::Application, callfn: GDConnectCallingFn) {
        let reconnect = match &callfn {
            GDConnectCallingFn::Add => false,
            _ => true,
        };
        let window = GoogledriveConnectWindow::new(&app, reconnect);

        let (sender, receiver) = async_channel::unbounded();
        let (shutdown_sender, shutdown_receiver) = async_channel::bounded(1);

        window.connect_connect(move |_| {
            runtime().spawn(
                clone!(@strong sender, @strong shutdown_receiver => async move {
                    let client = DragonDisplayGDClient::new(sender.clone());
                    client.connect(shutdown_receiver).await;
                }),
            );
        });

        window.connect_cancel(
            clone!(@weak self as obj, @weak app, @strong shutdown_sender => move |window| {
                window.destroy();
                try_emit!(obj, shutdown_sender.send_blocking(()).context(SendBackendSnafu), false);
                obj.select_window(&app);
            }),
        );

        DragonDisplayGDClient::connect_event(
            receiver,
            clone!(@weak self as obj, @weak window, @weak app => move |event| match event {
                GdClientEvent::Url { url } => {
                    window.update_url(&url);
                },
                GdClientEvent::Accesstoken { token } => {
                    window.destroy();
                    obj.imp().campaign.borrow().set_token(token);
                    match callfn {
                        GDConnectCallingFn::SyncFiles => obj.googledrive_synchronize(&app),
                        _ => obj.googledrive_loadfolders(&app)
                    }
                },
                GdClientEvent::Error { msg, fatal } => {
                    obj.emit_by_name::<()>("error", &[&msg, &fatal]);
                },
                _ => panic!("Invalid state"),
            }),
        );

        window.present();
    }

    /// Create and present the google drive load folders, a progress bar for loading all folders in
    /// the target drive
    pub fn googledrive_loadfolders(&self, app: &adw::Application) {
        let progbar = DdProgressBar::new("Indexing google drive folders".to_string());
        let window = Window::builder().application(app).child(&progbar).build();
        let new_folders: Rc<RefCell<Vec<GoogleFolderObject>>> = Rc::new(RefCell::new(Vec::new()));
        let (sender, receiver) = async_channel::unbounded();

        let token_snapshot = self.token();
        match &*self.imp().gd_client_state.borrow() {
            GdClientState::General => {
                runtime().spawn(clone!(@strong sender => async move {
                    let client = DragonDisplayGDClient::new(sender.clone());
                    client
                        .total_folders(token_snapshot)
                        .await;
                }));
            }
            GdClientState::ListFolders { folders, .. } => {
                progbar.update_total(folders.len());
                let folders_snapshot = folders.clone();
                runtime().spawn(clone!(@strong sender => async move {
                    let client = DragonDisplayGDClient::new(sender.clone());
                    client.list_folders(token_snapshot, folders_snapshot).await;
                }));
            }
            _ => panic!("Invalid state"),
        }

        DragonDisplayGDClient::connect_event(
            receiver,
            clone!(@weak self as obj, @weak progbar, @strong new_folders, @weak window, @weak app => move |event| {
                match event {
                    GdClientEvent::Totalfolders { total } => {
                        progbar.update_total(total);
                        let folder_snapshot = new_folders.borrow().clone();
                        let token_snapshot = obj.token();
                        obj.imp().gd_client_state.replace(GdClientState::ListFolders { folders: new_folders.take(), indexed_folders: Vec::new() });
                        runtime().spawn(clone!(@strong sender => async move {
                            DragonDisplayGDClient::new(sender.clone()).list_folders(token_snapshot, folder_snapshot).await;
                        }));
                    }
                    GdClientEvent::Folder { folder } => {
                        new_folders.borrow_mut().push(folder);
                    }
                    GdClientEvent::Childrenfolders { parent, children } => {
                        progbar.update_progress(children);
                        obj.imp().gd_client_state.borrow_mut().pop_folder();
                        obj.imp().gd_client_state.borrow_mut().new_indexed(parent);
                    }
                    GdClientEvent::Finished => {
                        window.destroy();
                        obj.googledrive_selectfolder(&app);
                    }
                    GdClientEvent::Refresh => {
                        let token_snapshot = obj.token();
                        runtime().spawn(clone!(@strong sender => async move {
                            DragonDisplayGDClient::new(sender.clone()).refresh_client(token_snapshot).await;
                        }));
                    }
                    GdClientEvent::Reconnect => {
                        window.destroy();
                        obj.googledrive_connect(&app, GDConnectCallingFn::LoadFolders);
                    }
                    GdClientEvent::Accesstoken { token: new_token } => {
                        obj.replace_token(new_token);
                        window.destroy();
                        obj.googledrive_loadfolders(&app);
                    }

                    _ => panic!("invalid event"),
                }

            }),
        );

        window.present();
    }

    /// Create and present the google drive select folder window
    pub fn googledrive_selectfolder(&self, app: &adw::Application) {
        let folders = self.imp().gd_client_state.borrow().indexed_folders();
        let window = DdGoogleFolderSelectWindow::new(app, folders);
        self.imp().gd_client_state.replace(GdClientState::General);

        window.connect_refresh(clone!(@weak self as obj, @weak app => move |window|{
            window.destroy();
            obj.googledrive_loadfolders(&app);
        }));

        window.connect_cancel(clone!(@weak self as obj, @weak app => move |window| {
            window.destroy();
            obj.select_window(&app);
        }));

        window.connect_choose(clone!(@weak self as obj, @weak app => move |window, id| {
            obj.imp().campaign.borrow().set_google_folder(id);
            try_emit!(obj, write_campaign_to_config(Campaign::from(&obj.imp().campaign.borrow())), false);
            window.destroy();
            obj.select_window(&app);
        }));

        window.present();
    }

    /// Synchronize a googledrive campaign
    pub fn googledrive_synchronize(&self, app: &adw::Application) {
        let progbar = DdProgressBar::new("Synchronizing files with google drive".to_string());
        let window = Window::builder().application(app).child(&progbar).build();
        let new_downloads: Rc<RefCell<Vec<(String, String)>>> = Rc::new(RefCell::new(Vec::new()));
        let (sender, receiver) = async_channel::unbounded();
        self.imp()
            .gd_client_state
            .replace_with(|state| match state {
                GdClientState::ListFolders { .. } => GdClientState::General,
                _ => state.clone(),
            });

        let path = self.imp().campaign.borrow().path();
        let token_snapshot = self.token();
        let folder_id = self
            .imp()
            .campaign
            .borrow()
            .sync_folder()
            .expect("Expected a folder");
        match &*self.imp().gd_client_state.borrow() {
            GdClientState::General => {
                runtime().spawn(clone!(@strong sender => async move {
                    let gd_client = DragonDisplayGDClient::new(sender.clone());
                    gd_client.get_and_remove(token_snapshot, folder_id, path).await;
                }));
            }
            GdClientState::DownloadFiles {
                total_needed,
                to_download,
            } => {
                progbar.update_total(*total_needed);
                progbar.update_progress(total_needed - to_download.len());
                let download_files = to_download.to_vec();
                runtime().spawn(clone!(@strong sender => async move {
                    let gd_client = DragonDisplayGDClient::new(sender.clone());
                    gd_client.download_files(token_snapshot, download_files, path).await;
                }));
            }
            _ => panic!("invalid state"),
        }

        DragonDisplayGDClient::connect_event(
            receiver,
            clone!(@weak self as obj, @weak window, @weak app => move |event| {
                match event {
                    GdClientEvent::Finished => {
                        obj.imp().gd_client_state.replace(GdClientState::General);
                        window.destroy();
                        if obj.imp().setup.get() {
                            obj.monitor_window(&app);
                        } else {
                            obj.imp()
                                .program
                                .get()
                                .expect("program should be set")
                                .update_grid();
                        }
                    },
                    GdClientEvent::FailedFiles { files } => {
                        let msg = format!("Failed to download the following files:\n{}", files.join(", "));
                        obj.emit_error(DragonDisplayError::Other { msg }, false);
                    }
                    GdClientEvent::DownloadFile { id, name } => {
                        new_downloads.borrow_mut().push((id, name));
                    },
                    GdClientEvent::Totalfolders { total } => {
                        progbar.update_total(total);
                        let current = total - new_downloads.borrow().len();
                        progbar.update_progress(current);
                        let to_download_snapshot = new_downloads.borrow().clone();
                        let token_snapshot = obj.token();
                        obj.imp().gd_client_state.replace(
                            GdClientState::DownloadFiles { total_needed: total, to_download: new_downloads.take() }
                        );
                        let path_snapshot = obj.imp().campaign.borrow().path();
                        runtime().spawn(clone!(@strong sender => async move {
                            let gd_client = DragonDisplayGDClient::new(sender.clone());
                            gd_client.download_files(token_snapshot, to_download_snapshot, path_snapshot).await;
                        }));

                    },
                    GdClientEvent::FileDownloaded => {
                        obj.imp().gd_client_state.borrow_mut().pop_folder();
                        progbar.update_progress(1);
                    },
                    GdClientEvent::Refresh => {
                        let token_snapshot = obj.imp().campaign.borrow().token().expect("Expected a token");
                        runtime().spawn(clone!(@strong sender => async move {
                            DragonDisplayGDClient::new(sender.clone()).refresh_client(token_snapshot).await;
                        }));
                    }
                    GdClientEvent::Reconnect => {
                        window.destroy();
                        obj.googledrive_connect(&app, GDConnectCallingFn::SyncFiles);
                    }
                    GdClientEvent::Accesstoken { token } => {
                        obj.replace_token(token);
                        window.destroy();
                        obj.googledrive_synchronize(&app);
                    },
                    _ => panic!("invalid event"),
                }
            }),
        );
        window.present();
    }

    /// Present the window to select a monitor
    pub fn monitor_window(&self, app: &adw::Application) {
        let window = try_emit!(self, SelectMonitorWindow::new(&app), true);

        window.connect_monitor(
            clone!(@weak self as obj, @weak app, @weak window => move |_, monitor| {
                window.destroy();
                obj.imp().monitor.set(monitor).expect("Expected monitor to not be set");
                obj.start_program(&app);
            }),
        );

        window.present();
    }

    /// This starts a new manager that will manage the two screens (control and display window)
    /// This manager keeps existing to handle possible refresh requests
    pub fn start_program(&self, app: &adw::Application) {
        let dragon_display = DragonDisplayProgram::new();

        dragon_display.run(
            &app,
            self.imp().monitor.get().expect("Expected a monitor"),
            self.imp().campaign.borrow().path(),
        );
        self.imp().setup.set(false);

        dragon_display.connect_refresh(clone!(@weak self as obj, @weak app => move |program| {
            let sync_option = obj.imp().campaign.borrow().sync_option();
            match sync_option {
                SynchronizationOption::None => program.update_grid(),
                SynchronizationOption::GoogleDrive { .. } => obj.googledrive_synchronize(&app),
            }
        }));

        self.imp()
            .program
            .set(dragon_display)
            .expect("program should not be set");
    }

    /**
     * ----------------------------------
     *
     * Signal connect functions
     *
     * --------------------------------
     **/

    /// Signal emitted when an error occurs
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

    /// A binding that gets the token of the current active campaign of the setup, panics if the
    /// token does not exist
    fn token(&self) -> Token {
        self.imp()
            .campaign
            .borrow()
            .token()
            .expect("Expected a token")
    }

    fn replace_token(&self, token: Token) {
        self.imp().campaign.borrow().set_token(token)
    }
}
