use async_channel::Sender;
use gtk::{
    gio::File,
    glib::{self},
    subclass::prelude::{ObjectSubclass, ObjectSubclassIsExt},
    MediaFile,
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
    #[template(resource = "/dragon/display/thumbnail_video.ui")]
    pub struct DdThumbnailVideo {
        #[template_child]
        pub button: TemplateChild<ToggleButton>,
        #[template_child]
        pub icon: TemplateChild<Picture>,
        #[template_child]
        pub label: TemplateChild<Label>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for DdThumbnailVideo {
        const NAME: &'static str = "DdThumbnailVideo";
        type Type = super::DdThumbnailVideo;
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
    impl ObjectImpl for DdThumbnailVideo {
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
    impl WidgetImpl for DdThumbnailVideo {}
}

glib::wrapper! {
    pub struct DdThumbnailVideo(ObjectSubclass<imp::DdThumbnailVideo>) @extends gtk::Widget,
            @implements gtk::Actionable, gtk::Accessible, gtk::Buildable,
                        gtk::ConstraintTarget;
}

impl DdThumbnailVideo {
    pub fn new(
        path: &PathBuf,
        sender: Sender<ControlWindowMessage>,
        prev_button: Option<&ToggleButton>,
    ) -> Self {
        let object = glib::Object::new::<Self>();
        let imp = object.imp();

        // the path.to_string was already checked
        let file_name = path
            .file_name()
            .expect("Could not get filename of file")
            .to_str()
            .expect("Could not obtain filename");
        imp.label.set_text(file_name);

        let file = File::for_path(path);
        let media_file = MediaFile::for_file(&file);
        imp.icon.set_paintable(Some(&media_file));
        imp.icon.set_content_fit(gtk::ContentFit::Fill);

        imp.button.set_group(prev_button);
        imp.button
            .connect_clicked(glib::clone!(@strong path => move |_| {
                sender
                    .send_blocking(ControlWindowMessage::Video {
                        video_path: path.clone(),
                    })
                    .expect("Channel closed");
            }));

        object
    }

    pub fn get_togglebutton(&self) -> &ToggleButton {
        let imp = self.imp();
        &imp.button
    }
}
