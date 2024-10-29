use async_channel::Sender;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::ObjectSubclassIsExt;

use crate::config::Campaign;
use crate::program_manager::ControlWindowMessage;
use crate::ui::control_window::Page;

use super::thumbnail_grid::DdThumbnailGrid;

mod imp {
    use async_channel::Sender;
    use std::cell::{Cell, RefCell};
    use std::io::{Error, ErrorKind};

    use glib::subclass::InitializingObject;
    use gtk::subclass::prelude::*;
    use gtk::{glib, Box, Button, CompositeTemplate, ToggleButton};
    use gtk::{prelude::*, template_callbacks};

    use crate::program_manager::ControlWindowMessage;
    use crate::widgets::thumbnail_grid::DdThumbnailGrid;

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dragon/display/image_page.ui")]
    pub struct DdImagePage {
        #[template_child]
        pub content: TemplateChild<Box>,
        pub thumbnail_grid: RefCell<Option<DdThumbnailGrid>>,
        pub sender: RefCell<Option<Sender<ControlWindowMessage>>>,
        pub fit: Cell<bool>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for DdImagePage {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "DdImagePage";
        type ParentType = gtk::Widget;
        type Type = super::DdImagePage;

        fn class_init(klass: &mut Self::Class) {
            Button::ensure_type();
            ToggleButton::ensure_type();

            klass.bind_template();
            klass.set_layout_manager_type::<gtk::BoxLayout>();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[template_callbacks]
    impl DdImagePage {
        #[template_callback]
        fn handle_rotate_90(&self, _: Button) {
            let sender = self.sender.borrow().clone().expect("No sender found");
            let grid = match self.thumbnail_grid.borrow().clone() {
                Some(g) => g,
                None => return,
            };
            let path = match grid.get_selected_image() {
                Some(p) => p,
                None => return,
            };
            let img = match image::open(&path) {
                Ok(img) => img,
                Err(_) => {
                    sender
                        .send_blocking(ControlWindowMessage::Error {
                            error: Error::new(
                                ErrorKind::NotFound,
                                "Current selected image does not exist anymore or the file extension and metadata do not match",
                            ),
                            fatal: false,
                        })
                        .expect("Channel closed");
                    return;
                }
            };
            let img = img.rotate90();
            match img.save(&path) {
                Ok(_) => sender
                    .send_blocking(ControlWindowMessage::Image { picture_path: path })
                    .expect("Channel closed"),
                Err(_) => sender
                    .send_blocking(ControlWindowMessage::Error {
                        error: Error::new(ErrorKind::WriteZero, "Could not save rotated image"),
                        fatal: false,
                    })
                    .expect("Channel closed"),
            }
        }

        #[template_callback]
        fn handle_rotate_180(&self, _: Button) {
            let sender = self.sender.borrow().clone().expect("No sender found");
            let grid = match self.thumbnail_grid.borrow().clone() {
                Some(g) => g,
                None => return,
            };
            let path = match grid.get_selected_image() {
                Some(p) => p,
                None => return,
            };
            let img = match image::open(&path) {
                Ok(img) => img,
                Err(_) => {
                    sender
                        .send_blocking(ControlWindowMessage::Error {
                            error: Error::new(
                                ErrorKind::NotFound,
                                "Current selected image does not exist anymore or the file extension and metadata do not match",
                            ),
                            fatal: false,
                        })
                        .expect("Channel closed");
                    return;
                }
            };
            let img = img.fliph();
            match img.save(&path) {
                Ok(_) => sender
                    .send_blocking(ControlWindowMessage::Image { picture_path: path })
                    .expect("Channel closed"),
                Err(_) => sender
                    .send_blocking(ControlWindowMessage::Error {
                        error: Error::new(ErrorKind::WriteZero, "Could not save rotated image"),
                        fatal: false,
                    })
                    .expect("Channel closed"),
            }
        }

        #[template_callback]
        fn handle_fit(&self, _: ToggleButton) {
            let sender = self.sender.borrow().clone().expect("No sender found");
            if self.fit.get() {
                self.fit.set(false);
                sender
                    .send_blocking(ControlWindowMessage::Fit { fit: false })
                    .expect("Channel closed");
            } else {
                self.fit.set(true);
                sender
                    .send_blocking(ControlWindowMessage::Fit { fit: true })
                    .expect("Channel closed");
            }
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for DdImagePage {
        fn constructed(&self) {
            // Call "constructed" on parent
            self.parent_constructed();
        }

        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for DdImagePage {}
}

glib::wrapper! {
    pub struct DdImagePage(ObjectSubclass<imp::DdImagePage>)
        @extends gtk::Widget,
            @implements gtk::Actionable, gtk::Accessible, gtk::Buildable,
                        gtk::ConstraintTarget;
}

impl DdImagePage {
    pub fn new(campaign: Campaign, sender: Sender<ControlWindowMessage>) -> Self {
        // set all properties
        let object = glib::Object::new::<Self>();
        let imp = object.imp();
        imp.sender.replace(Some(sender.clone()));
        let thumbnail_widget = DdThumbnailGrid::new(campaign, sender, Page::IMAGE);
        thumbnail_widget.set_halign(gtk::Align::Fill);
        thumbnail_widget.set_valign(gtk::Align::Fill);
        thumbnail_widget.set_hexpand(true);
        thumbnail_widget.set_vexpand(true);
        imp.content.append(&thumbnail_widget);
        imp.thumbnail_grid.replace(Some(thumbnail_widget));

        object
    }
}
