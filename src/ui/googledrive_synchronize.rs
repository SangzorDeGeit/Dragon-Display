use adw::Application;
use async_channel::Sender;
use gtk::prelude::*;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{gio, glib};
use std::io::Error;

use crate::dragon_display::setup::config::Campaign;
use crate::dragon_display::setup::google_drive::synchronize_files;
use crate::runtime;
use crate::widgets::progress_bar::DdProgressBar;

mod imp {

    use std::cell::RefCell;
    use std::io::Error;

    use async_channel::Sender;
    use glib::subclass::InitializingObject;
    use gtk::subclass::prelude::*;
    use gtk::{glib, Box, CompositeTemplate};

    use crate::dragon_display::setup::config::Campaign;

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dragon/display/googledrive_synchronize.ui")]
    pub struct GoogledriveSynchronizeWindow {
        #[template_child]
        pub main_box: TemplateChild<Box>,
        pub campaign: RefCell<Campaign>,
        pub sender: RefCell<Option<Sender<Result<(Campaign, Vec<String>), Error>>>>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for GoogledriveSynchronizeWindow {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "DdGoogledriveSynchronizeWindow";
        type Type = super::GoogledriveSynchronizeWindow;
        type ParentType = gtk::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for GoogledriveSynchronizeWindow {
        fn constructed(&self) {
            // Call "constructed" on parent
            self.parent_constructed();
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for GoogledriveSynchronizeWindow {}

    // Trait shared by all windows
    impl WindowImpl for GoogledriveSynchronizeWindow {}

    // Trait shared by all application windows
    impl ApplicationWindowImpl for GoogledriveSynchronizeWindow {}
}

glib::wrapper! {
    pub struct GoogledriveSynchronizeWindow(ObjectSubclass<imp::GoogledriveSynchronizeWindow>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
            @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                        gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl GoogledriveSynchronizeWindow {
    pub fn new(
        app: &Application,
        campaign: Campaign,
        sender: Sender<Result<(Campaign, Vec<String>), Error>>,
    ) -> Self {
        // set all properties
        let object = glib::Object::new::<Self>();
        object.set_property("application", app);
        let imp = object.imp();
        imp.campaign.replace(campaign);
        imp.sender.replace(Some(sender));
        Self::initialize(imp);

        object
    }

    /// this initialize function is called after the input variables for new() are set
    fn initialize(imp: &imp::GoogledriveSynchronizeWindow) {
        let campaign = imp.campaign.borrow().clone();
        let (progress_sender, progress_receiver) = async_channel::unbounded();
        let progress_bar = DdProgressBar::new(progress_receiver);
        imp.main_box.append(&progress_bar);
        let sender = imp.sender.borrow().clone().expect("No sender found");

        runtime().spawn(async move {
            let (new_campaign, failed_files) =
                match synchronize_files(campaign, progress_sender).await {
                    Ok((c, f)) => (c, f),
                    Err(e) => {
                        sender.send_blocking(Err(e)).expect("Channel closed");
                        return;
                    }
                };
            sender
                .send_blocking(Ok((new_campaign, failed_files)))
                .expect("Channel closed");
        });
    }
}
