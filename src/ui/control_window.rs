use adw::Application;
use async_channel::Sender;
use gtk::prelude::*;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{gio, glib};
use std::io::Error;

use crate::config::Campaign;
use crate::widgets::image_page::DdImagePage;

pub enum UpdateDisplayMessage {
    Image { picture_path: String },
    Refresh,
    Error { error: Error, fatal: bool },
}

mod imp {

    use std::cell::RefCell;

    use async_channel::Sender;
    use glib::subclass::InitializingObject;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::{glib, template_callbacks, Box, Button, CompositeTemplate, Stack, StackSwitcher};

    use super::UpdateDisplayMessage;

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
        pub sender: RefCell<Option<Sender<UpdateDisplayMessage>>>,
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
            self.sender
                .borrow()
                .clone()
                .expect("Sender not found")
                .send_blocking(UpdateDisplayMessage::Refresh)
                .expect("Channel closed");
        }

        #[template_callback]
        fn handle_options(&self, _: Button) {
            todo!("implement this function");
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
        sender: Sender<UpdateDisplayMessage>,
    ) -> Self {
        // set all properties
        let object = glib::Object::new::<Self>();
        object.set_property("application", app);
        let imp = object.imp();
        imp.sender.replace(Some(sender.clone()));
        imp.stackswitcher.set_stack(Some(&imp.stack));
        let image_page = DdImagePage::new(campaign, sender);
        image_page.set_halign(gtk::Align::Fill);
        image_page.set_valign(gtk::Align::Fill);
        image_page.set_hexpand(true);
        image_page.set_vexpand(true);
        imp.images.append(&image_page);

        object
    }
}
