use adw::Application;
use async_channel::Receiver;
use glib::spawn_future_local;
use gtk::gio::File;
use gtk::prelude::*;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{gio, glib, Picture, Video};

use crate::program_manager::DisplayWindowMessage;

mod imp {

    use std::cell::{Cell, RefCell};

    use glib::subclass::InitializingObject;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::{glib, Box, Button, CompositeTemplate};

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dragon/display/display_window.ui")]
    pub struct DdDisplayWindow {
        #[template_child]
        pub content: TemplateChild<Box>,
        pub fit: Cell<bool>,
        pub current_content: RefCell<String>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for DdDisplayWindow {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "DdDisplayWindow";
        type Type = super::DdDisplayWindow;
        type ParentType = gtk::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Button::ensure_type();

            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for DdDisplayWindow {
        fn constructed(&self) {
            // Call "constructed" on parent
            self.parent_constructed();
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for DdDisplayWindow {}

    // Trait shared by all windows
    impl WindowImpl for DdDisplayWindow {}

    // Trait shared by all application windows
    impl ApplicationWindowImpl for DdDisplayWindow {}
}

glib::wrapper! {
    pub struct DdDisplayWindow(ObjectSubclass<imp::DdDisplayWindow>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
            @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                        gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl DdDisplayWindow {
    pub fn new(app: &Application, receiver: Receiver<DisplayWindowMessage>) -> Self {
        // set all properties
        let object = glib::Object::new::<Self>();
        object.set_property("application", app);
        let imp = object.imp();
        Self::await_updates(imp, receiver);

        object
    }

    fn await_updates(imp: &imp::DdDisplayWindow, receiver: Receiver<DisplayWindowMessage>) {
        let content = imp.content.clone();
        let fit = imp.fit.clone();
        let current_content = imp.current_content.clone();
        spawn_future_local(async move {
            while let Ok(message) = receiver.recv().await {
                if let Some(child) = content.first_child() {
                    content.remove(&child);
                }
                match message {
                    DisplayWindowMessage::Image { picture_path } => {
                        current_content.replace(picture_path.clone());
                        let file = File::for_path(picture_path);
                        let image = Picture::builder().file(&file).build();
                        if fit.get() {
                            image.set_content_fit(gtk::ContentFit::Fill);
                        }
                        content.append(&image);
                    }
                    DisplayWindowMessage::Fit { fit: f } => {
                        fit.set(f);
                        let file = File::for_path(current_content.borrow().clone());
                        let image = Picture::builder().file(&file).build();
                        if f {
                            image.set_content_fit(gtk::ContentFit::Fill);
                        } else {
                            image.set_content_fit(gtk::ContentFit::Contain);
                        }
                        content.append(&image);
                    }
                    DisplayWindowMessage::Video { video_path } => {
                        current_content.replace(video_path.clone());
                        let file = File::for_path(video_path);
                        let video = Video::builder()
                            .loop_(true)
                            .autoplay(true)
                            .file(&file)
                            .sensitive(false)
                            .build();
                        content.append(&video);
                    }
                }
            }
        });
    }
}
