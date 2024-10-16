use crate::ui::control_window::UpdateDisplayMessage;
use async_channel::Sender;
use gtk::{
    glib::{self, clone},
    subclass::prelude::{ObjectSubclass, ObjectSubclassIsExt},
};
use gtk::{prelude::*, ToggleButton};
use std::fs::DirEntry;

use gtk::subclass::prelude::*;

mod imp {

    use glib::subclass::InitializingObject;
    use gtk::{CompositeTemplate, Image, Label, ToggleButton};

    use super::*;
    // Object holding the campaign
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dragon/display/thumbnail.ui")]
    pub struct DdThumbnail {
        #[template_child]
        pub button: TemplateChild<ToggleButton>,
        #[template_child]
        pub icon: TemplateChild<Image>,
        #[template_child]
        pub label: TemplateChild<Label>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for DdThumbnail {
        const NAME: &'static str = "DdThumbnail";
        type Type = super::DdThumbnail;
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
    impl ObjectImpl for DdThumbnail {
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
    impl WidgetImpl for DdThumbnail {}
}

glib::wrapper! {
    pub struct DdThumbnail(ObjectSubclass<imp::DdThumbnail>) @extends gtk::Widget,
            @implements gtk::Actionable, gtk::Accessible, gtk::Buildable,
                        gtk::ConstraintTarget;
}

impl DdThumbnail {
    pub fn new(
        file: &DirEntry,
        sender: Sender<UpdateDisplayMessage>,
        prev_button: Option<ToggleButton>,
    ) -> Self {
        let object = glib::Object::new::<Self>();
        let imp = object.imp();
        let file_name = file.file_name();
        let name = file_name.to_str().expect("File has no filename");
        let file_path = file.path();
        let path = file_path.to_str().expect("No file path found").to_owned();
        imp.icon.set_file(Some(path.as_str()));
        imp.label.set_text(name);
        imp.button.set_group(prev_button.as_ref());

        imp.button.connect_clicked(clone!(@strong path => move |_| {
            sender
                .send_blocking(UpdateDisplayMessage::Image {
                    picture_path: path.to_string(),
                })
                .expect("Channel closed");
        }));

        object
    }

    pub fn get_togglebutton(&self) -> ToggleButton {
        let imp = self.imp();
        imp.button.clone()
    }
}
