use gdk4::Texture;
use gtk::{
    gio::File,
    glib::{self},
    subclass::prelude::{ObjectSubclass, ObjectSubclassIsExt},
};
use gtk::{prelude::*, ToggleButton};
use std::path::PathBuf;

use gtk::subclass::prelude::*;

pub enum MediaType {
    Image,
    Video,
}

mod imp {

    use std::cell::RefCell;

    use glib::subclass::InitializingObject;
    use gtk::{CompositeTemplate, Label, Picture};

    use super::*;
    // Object holding the campaign
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dragon/display/thumbnail.ui")]
    pub struct DdThumbnail {
        #[template_child]
        pub icon: TemplateChild<Picture>,
        #[template_child]
        pub label: TemplateChild<Label>,
        pub path: RefCell<Option<String>>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for DdThumbnail {
        const NAME: &'static str = "DdThumbnail";
        type Type = super::DdThumbnail;
        type ParentType = gtk::ToggleButton;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_layout_manager_type::<gtk::BoxLayout>();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for DdThumbnail {
        fn constructed(&self) {
            self.parent_constructed();
        }

        fn dispose(&self) {
            println!("Dispose called on button {}", self.label.text());
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for DdThumbnail {}

    impl ButtonImpl for DdThumbnail {}

    impl ToggleButtonImpl for DdThumbnail {}
}

glib::wrapper! {
    pub struct DdThumbnail(ObjectSubclass<imp::DdThumbnail>)
        @extends gtk::ToggleButton, gtk::Button, gtk::Widget,
            @implements gtk::Actionable, gtk::Accessible, gtk::Buildable,
                        gtk::ConstraintTarget;
}

impl DdThumbnail {
    pub fn new(path: &PathBuf, prev_button: Option<&ToggleButton>, t: &MediaType) -> Self {
        let object = glib::Object::new::<Self>();
        let imp = object.imp();
        let file_name = path
            .file_name()
            .expect("Could not get filename of file")
            .to_str()
            .expect("Could not obtain filename");
        // the path.to_string was already checked
        let file_path = path
            .to_str()
            .expect("Path of file could not be obtained")
            .to_string();
        match t {
            MediaType::Image => {
                let image = Texture::from_filename(&file_path).expect("Expected an image");
                imp.icon.set_paintable(Some(&image));
                imp.label.set_text(file_name);
            }
            MediaType::Video => {
                let markup = format!("<span font=\"30\">{}</span>", &file_name);
                imp.label.set_label(&markup);
            }
        }
        imp.icon.set_content_fit(gtk::ContentFit::Fill);
        imp.path.replace(Some(file_path));

        object.set_group(prev_button);
        object
    }

    /// Returns the file path value linked to the button
    pub fn file(&self) -> String {
        let binding = self.imp().path.borrow().clone();
        binding.expect("filepath should be set")
    }

    /// Update the thumbnail with a new file, updating the picture and the name
    pub fn update(&self, new_file: &PathBuf) {
        let file_name = new_file
            .file_name()
            .expect("Could not get filename of file")
            .to_str()
            .expect("Could not obtain filename");
        // the path.to_string was already checked
        let file_path = new_file
            .to_str()
            .expect("Path of file could not be obtained")
            .to_string();
        let file = File::for_path(&file_path);
        self.imp().icon.set_file(Some(&file));
        self.imp().label.set_text(file_name);
        self.imp().path.replace(Some(file_path));
    }
}
