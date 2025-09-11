use adw::Application;
use async_channel::Receiver;
use gdk4::Texture;
use glib::spawn_future_local;
use gtk::gdk_pixbuf::{Pixbuf, PixbufRotation};
use gtk::gio::File;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{gio, glib, MediaFile, Picture};
use gtk::{prelude::*, Widget};

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
        pub current_rotation: Cell<u32>,
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
        let current_rotation = imp.current_rotation.clone();
        let media_file = MediaFile::new();
        current_rotation.set(0);
        spawn_future_local(async move {
            while let Ok(message) = receiver.recv().await {
                let child = match content.first_child() {
                    Some(child) => {
                        content.remove(&child);
                        child
                    }
                    None => Widget::from(Picture::new()),
                };
                match message {
                    DisplayWindowMessage::Image { picture_path } => {
                        current_content.replace(picture_path.clone());
                        let image = match child.downcast_ref::<Picture>() {
                            Some(image) => {
                                image.set_filename(Some(&picture_path));
                                image
                            }
                            None => {
                                let file = File::for_path(picture_path);
                                &Picture::builder().file(&file).build()
                            }
                        };
                        if fit.get() {
                            image.set_content_fit(gtk::ContentFit::Fill);
                        }
                        content.append(image);
                    }
                    DisplayWindowMessage::Fit { fit: f } => {
                        fit.set(f);
                        let child = match child.downcast_ref::<Picture>() {
                            Some(child) => child,
                            None => continue,
                        };

                        if f {
                            child.set_content_fit(gtk::ContentFit::Fill);
                        } else {
                            child.set_content_fit(gtk::ContentFit::Contain);
                        }
                        content.append(child);
                    }
                    DisplayWindowMessage::Rotate { rotation } => {
                        let child = match child.downcast_ref::<Picture>() {
                            Some(child) => child,
                            None => continue,
                        };
                        let pixbuf = match Pixbuf::from_file(current_content.borrow().clone()) {
                            Ok(p) => p,
                            Err(_) => continue,
                        };
                        let mut new_rotation = current_rotation.get();
                        new_rotation = match rotation {
                            PixbufRotation::None => new_rotation,
                            PixbufRotation::Counterclockwise => (new_rotation + 270) % 360,
                            PixbufRotation::Upsidedown => (new_rotation + 180) % 360,
                            PixbufRotation::Clockwise => (new_rotation + 90) % 360,
                            _ => new_rotation,
                        };
                        match new_rotation {
                            0 => pixbuf
                                .rotate_simple(PixbufRotation::None)
                                .expect("failed to rotate"),
                            90 => pixbuf
                                .rotate_simple(PixbufRotation::Clockwise)
                                .expect("failed to rotate"),
                            180 => pixbuf
                                .rotate_simple(PixbufRotation::Upsidedown)
                                .expect("failed to rotate"),
                            270 => pixbuf
                                .rotate_simple(PixbufRotation::Counterclockwise)
                                .expect("failed to rotate"),
                            _ => panic!("Invalid rotation"),
                        };
                        current_rotation.set(new_rotation);
                        let texture = Texture::for_pixbuf(&pixbuf);
                        child.set_paintable(Some(&texture));
                        content.append(child);
                    }
                    DisplayWindowMessage::Video { video_path } => {
                        let current_path = video_path
                            .to_str()
                            .expect("Could not obtain path")
                            .to_string();
                        current_content.replace(current_path);
                        let file = File::for_path(video_path);
                        media_file.set_file(Some(&file));
                        let video = match child.downcast_ref::<Picture>() {
                            Some(image) => {
                                image.set_paintable(Some(&media_file));
                                image
                            }
                            None => &Picture::builder().paintable(&media_file).build(),
                        };
                        media_file.play();
                        media_file.set_loop(true);
                        content.append(video);
                    }
                }
            }
        });
    }
}
