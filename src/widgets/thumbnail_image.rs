use async_channel::Sender;
use gtk::{
    gio::File,
    glib::{self, clone},
    subclass::prelude::{ObjectSubclass, ObjectSubclassIsExt},
};
use gtk::{prelude::*, ToggleButton};
use std::fs::DirEntry;

use gtk::subclass::prelude::*;

use crate::program_manager::ControlWindowMessage;

mod imp {

    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use glib::subclass::InitializingObject;
    use gtk::{CompositeTemplate, Label, Picture, ToggleButton};

    use super::*;
    // Object holding the campaign
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dragon/display/thumbnail_image.ui")]
    pub struct DdThumbnailImage {
        #[template_child]
        pub button: TemplateChild<ToggleButton>,
        #[template_child]
        pub icon: TemplateChild<Picture>,
        #[template_child]
        pub label: TemplateChild<Label>,
        pub selected: Rc<Cell<bool>>,
        pub path: RefCell<String>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for DdThumbnailImage {
        const NAME: &'static str = "DdThumbnailImage";
        type Type = super::DdThumbnailImage;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_layout_manager_type::<gtk::BoxLayout>();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for DdThumbnailImage {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for DdThumbnailImage {}
}

glib::wrapper! {
    pub struct DdThumbnailImage(ObjectSubclass<imp::DdThumbnailImage>) @extends gtk::Widget,
            @implements gtk::Actionable, gtk::Accessible, gtk::Buildable,
                        gtk::ConstraintTarget;
}

impl DdThumbnailImage {
    pub fn new(
        file: &DirEntry,
        sender: Sender<ControlWindowMessage>,
        prev_button: Option<ToggleButton>,
    ) -> Self {
        let object = glib::Object::new::<Self>();
        let imp = object.imp();
        let file_name = file.file_name();
        let name = file_name.to_str().expect("File has no filename");
        let file_path = file.path();
        let path = file_path.to_str().expect("No file path found").to_owned();
        let file = File::for_path(path.as_str());
        imp.icon.set_file(Some(&file));
        imp.label.set_text(name);
        imp.button.set_group(prev_button.as_ref());
        imp.path.replace(path.clone());

        let selected = imp.selected.clone();
        imp.button
            .connect_toggled(clone!(@strong path => move |button| {
                if button.is_active() {
                    selected.set(true);
                sender
                    .send_blocking(ControlWindowMessage::Image {
                        picture_path: path.to_string(),
                    })
                    .expect("Channel closed");
                } else {
                    selected.set(false);
                }
            }));

        object
    }

    pub fn get_togglebutton(&self) -> ToggleButton {
        let imp = self.imp();
        imp.button.clone()
    }

    pub fn get_path(&self) -> String {
        let imp = self.imp();
        imp.path.borrow().clone()
    }

    pub fn selected(&self) -> bool {
        let imp = self.imp();
        imp.selected.get()
    }
}
