use async_channel::Sender;
use gtk::{
    gio::File,
    glib::{self},
    subclass::prelude::{ObjectSubclass, ObjectSubclassIsExt},
};
use gtk::{prelude::*, ToggleButton};
use std::path::PathBuf;

use gtk::subclass::prelude::*;

use crate::program_manager::ControlWindowMessage;

mod imp {

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
        path: &PathBuf,
        sender: Sender<ControlWindowMessage>,
        prev_button: Option<&ToggleButton>,
    ) -> Self {
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
        let file = File::for_path(&file_path);
        imp.icon.set_file(Some(&file));
        imp.icon.set_content_fit(gtk::ContentFit::Fill);
        imp.label.set_text(file_name);
        imp.button.set_group(prev_button);

        imp.button.connect_clicked(move |_| {
            sender
                .send_blocking(ControlWindowMessage::Image {
                    picture_path: file_path.clone(),
                })
                .expect("Channel closed");
        });

        object
    }

    pub fn get_togglebutton(&self) -> &ToggleButton {
        let imp = self.imp();
        &imp.button
    }
}
