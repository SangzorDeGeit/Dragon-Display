use adw::Application;
use async_channel::Sender;
use gtk::prelude::*;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{gio, glib};

use crate::config::Campaign;
use crate::program_manager::ControlWindowMessage;
use crate::widgets::image_page::DdImagePage;
use crate::widgets::video_page::DdVideoPage;

pub enum Page {
    IMAGE,
    VIDEO,
}

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
                self.obj().refresh_widgets();
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
                    obj.refresh_widgets();
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
                    obj.refresh_widgets();
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
        let image_page = DdImagePage::new(campaign.clone(), sender.clone());
        image_page.set_halign(gtk::Align::Fill);
        image_page.set_valign(gtk::Align::Fill);
        image_page.set_hexpand(true);
        image_page.set_vexpand(true);
        imp.images.append(&image_page);
        let video_page = DdVideoPage::new(campaign, sender);
        video_page.set_halign(gtk::Align::Fill);
        video_page.set_valign(gtk::Align::Fill);
        video_page.set_hexpand(true);
        video_page.set_vexpand(true);
        imp.videos.append(&video_page);

        object
    }

    pub fn refresh_widgets(&self) {
        let imp = self.imp();
        let sender = imp.sender.borrow().clone().expect("No sender found");
        let image_page = DdImagePage::new(imp.campaign.borrow().clone(), sender);
        image_page.set_halign(gtk::Align::Fill);
        image_page.set_valign(gtk::Align::Fill);
        image_page.set_hexpand(true);
        image_page.set_vexpand(true);
        imp.images
            .remove(&imp.images.first_child().expect("No image page found"));
        imp.images.append(&image_page);

        // create new video page append it to the video box

        // create new vtt page append it to the vtt box
    }
}
