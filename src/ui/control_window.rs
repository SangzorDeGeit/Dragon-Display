use std::fs::read_dir;
use std::path::PathBuf;

use adw::Application;
use async_channel::Sender;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{gio, glib};
use gtk::{prelude::*, Label};

use crate::config::{Campaign, IMAGE_EXTENSIONS, VIDEO_EXTENSIONS};
use crate::program_manager::ControlWindowMessage;
use crate::widgets::image_page::DdImagePage;
use crate::widgets::video_page::DdVideoPage;

mod imp {

    use std::cell::RefCell;

    use async_channel::Sender;
    use glib::subclass::InitializingObject;
    use gtk::glib::spawn_future_local;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::{glib, template_callbacks, Box, Button, CompositeTemplate, Stack, StackSwitcher};

    use crate::config::Campaign;
    use crate::config::SynchronizationOption;
    use crate::program_manager::ControlWindowMessage;

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dragon/display/control_window.ui")]
    pub struct ControlWindow {
        #[template_child]
        pub stack: TemplateChild<Stack>,
        #[template_child]
        pub stackswitcher: TemplateChild<StackSwitcher>,
        #[template_child]
        pub images: TemplateChild<Box>,
        #[template_child]
        pub videos: TemplateChild<Box>,
        #[template_child]
        pub vtts: TemplateChild<Box>,
        pub campaign: RefCell<Campaign>,
        pub sender: RefCell<Option<Sender<ControlWindowMessage>>>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for ControlWindow {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "DdControlWindow";
        type Type = super::ControlWindow;
        type ParentType = gtk::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Button::ensure_type();

            klass.bind_template();
            klass.bind_template_callbacks()
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[template_callbacks]
    impl ControlWindow {
        #[template_callback]
        fn handle_refresh(&self, _: Button) {
            if matches!(
                self.campaign.borrow().sync_option,
                SynchronizationOption::None
            ) {
                self.obj().update_widgets();
                return;
            }
            // make async channel
            let (refresh_sender, refresh_receiver) = async_channel::bounded(1);
            // send refresh signal
            self.sender
                .borrow()
                .clone()
                .expect("Sender not found")
                .send_blocking(ControlWindowMessage::Refresh {
                    sender: refresh_sender,
                })
                .expect("Channel closed");
            // await message
            let obj = self.obj().clone();
            spawn_future_local(async move {
                while let Ok(_) = refresh_receiver.recv().await {
                    obj.update_widgets();
                }
            });
        }

        #[template_callback]
        fn handle_options(&self, _: Button) {
            let (refresh_sender, refresh_receiver) = async_channel::bounded(1);
            self.sender
                .borrow()
                .clone()
                .expect("No sender found")
                .send_blocking(ControlWindowMessage::Options {
                    sender: refresh_sender,
                })
                .expect("Channel closed");
            let obj = self.obj().clone();
            spawn_future_local(async move {
                while let Ok(_) = refresh_receiver.recv().await {
                    obj.update_widgets();
                }
            });
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for ControlWindow {
        fn constructed(&self) {
            // Call "constructed" on parent
            self.parent_constructed();
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for ControlWindow {}

    // Trait shared by all windows
    impl WindowImpl for ControlWindow {}

    // Trait shared by all application windows
    impl ApplicationWindowImpl for ControlWindow {}
}

glib::wrapper! {
    pub struct ControlWindow(ObjectSubclass<imp::ControlWindow>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
            @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                        gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl ControlWindow {
    pub fn new(
        app: &Application,
        campaign: Campaign,
        sender: Sender<ControlWindowMessage>,
    ) -> Self {
        // set all properties
        let object = glib::Object::new::<Self>();
        object.set_property("application", app);
        let imp = object.imp();
        imp.campaign.replace(campaign.clone());
        imp.sender.replace(Some(sender.clone()));
        imp.stackswitcher.set_stack(Some(&imp.stack));
        object.update_widgets();

        object
    }

    pub fn update_widgets(&self) {
        let imp = self.imp();
        let sender = imp.sender.borrow().clone().expect("No sender found");
        let files = match read_dir(&imp.campaign.borrow().path) {
            Ok(f) => f,
            Err(e) => {
                sender
                    .send_blocking(ControlWindowMessage::Error {
                        error: e,
                        fatal: true,
                    })
                    .expect("Channel closed");
                return;
            }
        };
        if let Some(current_page) = imp.images.first_child() {
            imp.images.remove(&current_page);
        }
        let (image_files, non_image_files): (Vec<PathBuf>, Vec<PathBuf>) = files
            .filter_map(|f| f.ok())
            .map(|f| f.path())
            .filter(|f| f.to_str().is_some() && f.extension().is_some())
            .filter(|f| f.extension().unwrap().to_str().is_some())
            .partition(|f| IMAGE_EXTENSIONS.contains(&f.extension().unwrap().to_str().unwrap()));
        if image_files.len() == 0 {
            let label = Label::builder()
                .label("You do not have any images")
                .halign(gtk::Align::Center)
                .valign(gtk::Align::Center)
                .hexpand(true)
                .vexpand(true)
                .build();
            imp.images.append(&label);
        } else {
            let image_page = DdImagePage::new(sender.clone(), image_files);
            image_page.set_halign(gtk::Align::Fill);
            image_page.set_valign(gtk::Align::Fill);
            image_page.set_hexpand(true);
            image_page.set_vexpand(true);
            imp.images.append(&image_page);
        }

        // create new video page append it to the video box
        if let Some(current_page) = imp.videos.first_child() {
            imp.videos.remove(&current_page);
        }
        let (video_files, _non_video_files): (Vec<PathBuf>, Vec<PathBuf>) = non_image_files
            .into_iter()
            .partition(|f| VIDEO_EXTENSIONS.contains(&f.extension().unwrap().to_str().unwrap()));
        if video_files.len() == 0 {
            let label = Label::builder()
                .label("You do not have any videos. Please be aware that adding videos could decrease performance of the program. Videos are not recommended when running on weak systems")
                .justify(gtk::Justification::Center)
                .halign(gtk::Align::Center)
                .valign(gtk::Align::Center)
                .hexpand(true)
                .vexpand(true)
                .wrap(true)
                .wrap_mode(gtk::pango::WrapMode::Word)
                .max_width_chars(40)
                .build();
            imp.videos.append(&label);
        } else {
            let video_page = DdVideoPage::new(sender, video_files);
            video_page.set_halign(gtk::Align::Fill);
            video_page.set_valign(gtk::Align::Fill);
            video_page.set_hexpand(true);
            video_page.set_vexpand(true);
            imp.videos.append(&video_page);
        }

        // create new vtt page append it to the vtt box
    }
}
