use std::collections::HashMap;

use adw::Application;
use async_channel::Sender;
use gtk::glib::{clone, closure_local, spawn_future_local};
use gtk::prelude::ObjectExt;
use gtk::prelude::*;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{gio, glib};

use crate::config::Campaign;
use crate::google_drive::{get_folder_amount, get_folder_tree, FolderResult};
use crate::runtime;
use crate::setup_manager::AddRemoveMessage;
use crate::widgets::google_folder_tree::DdGoogleFolderTree;
use crate::widgets::progress_bar::DdProgressBar;
use crate::widgets::progress_bar::ProgressMessage;

mod imp {

    use async_channel::Sender;
    use gtk::prelude::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    use glib::subclass::InitializingObject;
    use gtk::subclass::prelude::*;
    use gtk::{glib, template_callbacks, Box, Button, CompositeTemplate, Label};

    use crate::config::Campaign;
    use crate::setup_manager::AddRemoveMessage;

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dragon/display/googledrive_select_folder.ui")]
    pub struct DdGoogleFolderSelectWindow {
        #[template_child]
        pub message_label: TemplateChild<Label>,
        #[template_child]
        pub selection_label: TemplateChild<Label>,
        #[template_child]
        pub load_select_widget: TemplateChild<Box>,
        #[template_child]
        pub choose_button: TemplateChild<Button>,
        #[template_child]
        pub refresh_button: TemplateChild<Button>,
        pub sender: RefCell<Option<Sender<AddRemoveMessage>>>,
        pub campaign: Rc<RefCell<Campaign>>,
        pub selected_id: Rc<RefCell<String>>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for DdGoogleFolderSelectWindow {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "DdGoogleFolderSelectWindow";
        type Type = super::DdGoogleFolderSelectWindow;
        type ParentType = gtk::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks()
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[template_callbacks]
    impl DdGoogleFolderSelectWindow {
        #[template_callback]
        fn handle_cancel(&self, _: Button) {
            self.sender
                .borrow()
                .clone()
                .expect("No sender found")
                .send_blocking(AddRemoveMessage::Cancel)
                .expect("Channel closed");
        }

        #[template_callback]
        fn handle_refresh(&self, button: Button) {
            self.load_select_widget.remove(
                &self
                    .load_select_widget
                    .first_child()
                    .expect("No child found"),
            );
            self.choose_button.set_sensitive(false);
            button.set_sensitive(false);
            self.selected_id.replace("".to_string());
            super::DdGoogleFolderSelectWindow::initialize(self);
        }

        #[template_callback]
        fn handle_choose(&self, _: Button) {
            let old_campaign = self.campaign.borrow().clone();
            let (name, path, access, refresh, _) = old_campaign.get_campaign_data();
            let new_campaign = Campaign::new_googledrive(
                name,
                path,
                access,
                refresh,
                self.selected_id.borrow().clone(),
            );
            self.sender
                .borrow()
                .clone()
                .expect("No sender found")
                .send_blocking(AddRemoveMessage::Campaign {
                    campaign: new_campaign,
                })
                .expect("Channel closed");
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for DdGoogleFolderSelectWindow {
        fn constructed(&self) {
            // Call "constructed" on parent
            self.parent_constructed();
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for DdGoogleFolderSelectWindow {}

    // Trait shared by all windows
    impl WindowImpl for DdGoogleFolderSelectWindow {}

    // Trait shared by all application windows
    impl ApplicationWindowImpl for DdGoogleFolderSelectWindow {}
}

glib::wrapper! {
    pub struct DdGoogleFolderSelectWindow(ObjectSubclass<imp::DdGoogleFolderSelectWindow>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
            @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                        gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl DdGoogleFolderSelectWindow {
    pub fn new(app: &Application, campaign: Campaign, sender: Sender<AddRemoveMessage>) -> Self {
        // set all properties
        let object = glib::Object::new::<Self>();
        object.set_property("application", app);
        let imp = object.imp();
        imp.campaign.replace(campaign);
        imp.sender.replace(Some(sender));

        Self::initialize(&imp);
        object
    }

    /// this initialize function is called after the input variables for new() are set
    pub fn initialize(imp: &imp::DdGoogleFolderSelectWindow) {
        // initialize all the variables needed
        let (progress_sender, progress_receiver) = async_channel::unbounded();
        let (result_sender, result_receiver) = async_channel::bounded(1);
        let sender = imp.sender.borrow().clone().expect("No sender found");
        let message_label = imp.message_label.clone();
        let refresh_button = imp.refresh_button.clone();
        let choose_button = imp.choose_button.clone();
        let selected_id = imp.selected_id.clone();
        let campaign = imp.campaign.clone();
        let load_select_widget = imp.load_select_widget.clone();
        let selection_label = imp.selection_label.clone();
        let (name, path, access_token, refresh_token, _) =
            imp.campaign.borrow().get_campaign_data();
        // set the label text
        message_label.set_text("Dragon Display is getting your google drive folder data");
        imp.selection_label.set_text("");

        // Create and add the progressbar
        let progress_bar = DdProgressBar::new(progress_receiver);
        imp.load_select_widget.append(&progress_bar);

        // await for result from backend process
        spawn_future_local(
            clone!(@strong access_token, @strong refresh_token => async move {
                while let Ok(result) = result_receiver.recv().await {
                    let new_campaign = Campaign::new_googledrive(
                        name.clone(),
                        path.clone(),
                        access_token.clone(),
                        refresh_token.clone(),
                        "".to_string(),
                    );
                    refresh_button.set_sensitive(true);
                    campaign.replace(new_campaign);
                    let folder_tree = DdGoogleFolderTree::new(result);
                    load_select_widget.remove(
                        &load_select_widget
                            .first_child()
                            .expect("No progressbar found"),
                    );
                    load_select_widget.append(&folder_tree);
                    message_label
                        .set_text("Select a folder where Dragon Display will download the images from. Current location:");
                    folder_tree.connect_closure(
                        "folder-selection-changed",
                        false,
                        closure_local!(@strong selected_id, @strong selection_label, @strong choose_button,  @strong refresh_button => move |_: DdGoogleFolderTree, name: String, id: String| {
                            selected_id.replace(id);
                            selection_label.set_text(&name);
                            choose_button.set_sensitive(true);
                            refresh_button.set_sensitive(true);
                        }),
                    );
                }
            }),
        );

        // call backend process
        Self::call_backend_process(
            sender,
            progress_sender,
            result_sender,
            access_token,
            refresh_token,
        );
    }

    fn call_backend_process(
        sender: Sender<AddRemoveMessage>,
        progress_sender: Sender<ProgressMessage>,
        result_sender: Sender<FolderResult>,
        access_token: String,
        refresh_token: String,
    ) {
        runtime().spawn(async move {
            let (total, access_token, refresh_token) =
                match get_folder_amount(access_token, refresh_token).await {
                    Ok(r) => r,
                    Err(e) => {
                        sender
                            .send_blocking(AddRemoveMessage::Error {
                                error: e,
                                fatal: false,
                            })
                            .expect("Channel closed");
                        return;
                    }
                };
            progress_sender
                .send_blocking(ProgressMessage::Total { amount: total })
                .expect("Channel closed");

            let mut id_name_map = HashMap::new();
            id_name_map.insert("root".to_string(), "My Drive".to_string());
            let id_child_map = HashMap::new();
            let folder_result = FolderResult {
                id_name_map,
                id_child_map,
                access_token,
                refresh_token,
            };

            let result =
                match get_folder_tree(folder_result, "root".to_string(), progress_sender).await {
                    Ok(r) => r,
                    Err(e) => {
                        sender
                            .send_blocking(AddRemoveMessage::Error {
                                error: e,
                                fatal: false,
                            })
                            .expect("Channel closed");
                        return;
                    }
                };
            result_sender.send_blocking(result).expect("Channel closed");
        });
    }
}
