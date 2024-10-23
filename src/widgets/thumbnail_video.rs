use async_channel::Sender;
use gdk_pixbuf::Pixbuf;
use gtk::gdk_pixbuf;
use gtk::{
    glib::{self, clone},
    subclass::prelude::{ObjectSubclass, ObjectSubclassIsExt},
};
use gtk::{prelude::*, ToggleButton};
use std::{fs::DirEntry, io::Error};

use gtk::subclass::prelude::*;

use gstreamer::prelude::*;
use gstreamer::{parse, prelude::Cast};
use gstreamer_app::AppSink;

use crate::program_manager::ControlWindowMessage;

mod imp {

    use glib::subclass::InitializingObject;
    use gtk::{CompositeTemplate, Image, Label, ToggleButton};

    use super::*;
    // Object holding the campaign
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dragon/display/thumbnail_video.ui")]
    pub struct DdThumbnailVideo {
        #[template_child]
        pub button: TemplateChild<ToggleButton>,
        #[template_child]
        pub icon: TemplateChild<Image>,
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
        imp.label.set_text(name);
        imp.button.set_group(prev_button.as_ref());

        let pixbuf = create_thumbnail(&path, sender.clone());
        imp.icon.set_from_pixbuf(pixbuf.as_ref());

        imp.button.connect_clicked(clone!(@strong path => move |_| {
            sender
                .send_blocking(ControlWindowMessage::Video {
                    video_path: path.to_string(),
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

pub fn create_thumbnail(path: &str, sender: Sender<ControlWindowMessage>) -> Option<Pixbuf> {
    // USE gstreamer to create the thumbnail image
    gstreamer::init().expect("Failed to initialize gstreamer");
    // make sure the pipeline format works with names that contain spaces
    let pipeline_friendly_path = path.replace(" ", r"\ ");
    let pipeline_string = format!("filesrc location={} ! decodebin ! videoconvert ! video/x-raw,format=RGB ! videoscale ! videorate ! video/x-raw,framerate=1/1 ! appsink name=sink", pipeline_friendly_path);
    let pipeline = match parse::launch(&pipeline_string) {
        Ok(e) => e,
        Err(e) => {
            println!("error: {}", e);
            sender
                .send_blocking(ControlWindowMessage::Error {
                    error: Error::new(
                        std::io::ErrorKind::BrokenPipe,
                        format!("Could not create a thumbnail for file: {} ", path),
                    ),
                    fatal: false,
                })
                .expect("Channel closed");
            return None;
        }
    };
    pipeline
        .set_state(gstreamer::State::Playing)
        .expect("Could not start playing state");

    let appsink = pipeline
        .dynamic_cast::<gstreamer::Bin>()
        .expect("Could not cast pipeline to gstreamer bin")
        .by_name("sink")
        .expect("No element of name sink")
        .dynamic_cast::<AppSink>()
        .expect("Could not cast to AppSink");

    // Get a sample from the sink (end of the pipeline)
    let sample = appsink.pull_sample().expect("Could not pull sample");
    // get the raw video data from the sample
    let buffer = sample.buffer().expect("Could not get sample buffer");
    // get the metadata from the sample
    let caps = sample.caps().expect("Could not get sample caps");
    // get indexed metadata that contains the width and height
    let video_info = gstreamer_video::VideoInfo::from_caps(caps).expect("Could not get video info");
    let width = video_info.width() as i32;
    let height = video_info.height() as i32;
    let stride = video_info
        .stride()
        .get(0)
        .expect("Stride array did not contain any values");
    let map = buffer
        .map_readable()
        .expect("Could not convert raw video data into readable");
    let data = map.as_slice();

    return Some(Pixbuf::from_mut_slice(
        data.to_vec(),
        gdk_pixbuf::Colorspace::Rgb,
        false,
        8,
        width,
        height,
        *stride,
    ));
}
