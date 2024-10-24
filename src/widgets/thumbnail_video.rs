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

        match create_thumbnail(&path) {
            Ok(p) => imp.icon.set_from_pixbuf(Some(&p)),
            Err(e) => {
                let errormsg = format!(
                    "Failed to create thumbnail for {}: {}",
                    &path,
                    e.to_string()
                );
                sender
                    .send_blocking(ControlWindowMessage::Error {
                        error: Error::new(std::io::ErrorKind::InvalidData, errormsg),
                        fatal: false,
                    })
                    .expect("Channel closed");
            }
        };

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

pub fn create_thumbnail(path: &str) -> Result<Pixbuf, Error> {
    // USE gstreamer to create the thumbnail image
    gstreamer::init().expect("Failed to initialize gstreamer");
    // make sure the pipeline format works with names that contain spaces
    let pipeline_friendly_path = path.replace(" ", r"\ ");
    let pipeline_string = format!("filesrc location={} ! decodebin ! videoconvert ! video/x-raw,format=RGB ! videoscale ! videorate ! video/x-raw,framerate=1/1 ! appsink name=sink", pipeline_friendly_path);
    let pipeline = match parse::launch(&pipeline_string) {
        Ok(e) => e,
        Err(e) => return Err(Error::new(std::io::ErrorKind::BrokenPipe, e.to_string())),
    };
    match pipeline.set_state(gstreamer::State::Playing) {
        Ok(_) => (),
        Err(e) => return Err(Error::new(std::io::ErrorKind::BrokenPipe, e.to_string())),
    }

    let appsink = pipeline
        .dynamic_cast::<gstreamer::Bin>()
        .expect("Could not cast pipeline to gstreamer bin")
        .by_name("sink")
        .expect("No element of name sink")
        .dynamic_cast::<AppSink>()
        .expect("Could not cast to AppSink");

    // Get a sample from the sink (end of the pipeline)
    let sample = match appsink.pull_sample() {
        Ok(s) => s,
        Err(e) => return Err(Error::new(std::io::ErrorKind::BrokenPipe, e.to_string())),
    };
    // get the raw video data from the sample
    let buffer = match sample.buffer() {
        Some(b) => b,
        None => {
            return Err(Error::new(
                std::io::ErrorKind::BrokenPipe,
                "Could not get video data",
            ))
        }
    };
    // get the metadata from the sample
    let caps = match sample.caps() {
        Some(c) => c,
        None => {
            return Err(Error::new(
                std::io::ErrorKind::BrokenPipe,
                "Could not get metadata",
            ))
        }
    };
    // get all video frame properties
    let video_info = match gstreamer_video::VideoInfo::from_caps(caps) {
        Ok(v) => v,
        Err(e) => return Err(Error::new(std::io::ErrorKind::BrokenPipe, e.to_string())),
    };
    let width = video_info.width() as i32;
    let height = video_info.height() as i32;
    let stride = match video_info.stride().get(0) {
        Some(s) => *s,
        None => 3 * width as i32,
    };

    let map = match buffer.map_readable() {
        Ok(m) => m,
        Err(_) => {
            return Err(Error::new(
                std::io::ErrorKind::BrokenPipe,
                "Could not convert raw video data into readable",
            ))
        }
    };
    let data = map.as_slice();

    return Ok(Pixbuf::from_mut_slice(
        data.to_vec(),
        gdk_pixbuf::Colorspace::Rgb,
        false,
        8,
        width,
        height,
        stride,
    ));
}
