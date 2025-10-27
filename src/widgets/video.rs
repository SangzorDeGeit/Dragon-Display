use async_channel::Sender;
/// A custom video widget that uses gstreamer pipelines to render videos and thumbnails
use gstreamer::{prelude::*, ClockTime};
use gstreamer::{Caps, Element, ElementFactory, Pipeline};
use gstreamer_app::AppSink;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::glib::spawn_future_local;
use gtk::glib::{self};
use gtk::subclass::prelude::ObjectSubclassIsExt;

use crate::runtime;

mod imp {

    use std::cell::Cell;

    use glib::subclass::InitializingObject;
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    // Object holding the state
    #[derive(Default)]
    pub struct DdVideoPipeline {
        pub height: Cell<i32>,
        pub width: Cell<i32>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for DdVideoPipeline {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "DdVideoPipeline";
        type ParentType = gtk::Widget;
        type Type = super::DdVideoPipeline;

        fn class_init(klass: &mut Self::Class) {
            klass.set_layout_manager_type::<gtk::BoxLayout>();
        }

        fn instance_init(_: &InitializingObject<Self>) {}
    }

    // Trait shared by all GObjects
    impl ObjectImpl for DdVideoPipeline {
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
    impl WidgetImpl for DdVideoPipeline {}
}

glib::wrapper! {
    pub struct DdVideoPipeline(ObjectSubclass<imp::DdVideoPipeline>)
        @extends gtk::Widget,
            @implements gtk::Actionable, gtk::Accessible, gtk::Buildable,
                        gtk::ConstraintTarget;
}

impl DdVideoPipeline {
    /// Create a new thumbnail from a video file, this acts as a static texture and ensures the
    /// pipeline is freed as soon as the texture is obtained
    pub fn thumbnail(path: &str) -> Pixbuf {
        gstreamer::init().expect("Failed to init gstreamer");
        let (pipeline, appsink) = Self::create_pipeline(path);
        pipeline
            .set_state(gstreamer::State::Playing)
            .expect("Could not set state to playing");
        let sample = appsink.pull_preroll().expect("Could not get sample");
        let buffer = sample.buffer().expect("Could not get sample buffer");
        let caps = sample.caps().expect("Could not get caps");
        let structure = caps.structure(0).expect("Could not get caps structure");
        let width = structure.get::<i32>("width").expect("Could not get width");
        let height = structure
            .get::<i32>("height")
            .expect("Could not get height");
        let data = buffer
            .map_readable()
            .expect("Could not get map")
            .as_slice()
            .to_vec();
        pipeline
            .set_state(gstreamer::State::Null)
            .expect("Could not set state to null");
        Pixbuf::from_mut_slice(
            data,
            gtk::gdk_pixbuf::Colorspace::Rgb,
            false,
            8,
            width,
            height,
            width * 3,
        )
    }

    /// Create a new playing video keeping the pipeline alive as long as the video is playing
    pub fn new_playing(path: &str, sender: Sender<Vec<u8>>) -> Self {
        let obj = glib::Object::new::<Self>();
        let (pipeline, appsink) = Self::create_pipeline(path);
        pipeline
            .set_state(gstreamer::State::Playing)
            .expect("Could not set state to playing");
        let sample = appsink.pull_preroll().expect("Could not pull first sample");
        let caps = sample.caps().expect("Could not get caps");
        let structure = caps.structure(0).expect("Could not get caps structure");
        let width = structure.get::<i32>("width").expect("Could not get width");
        let height = structure
            .get::<i32>("height")
            .expect("Could not get height");
        obj.imp().height.set(height);
        obj.imp().width.set(width);

        runtime().spawn(async move {
            while !appsink.is_eos() {
                if let Some(sample) = appsink.try_pull_sample(ClockTime::from_useconds(100)) {
                    let buffer = sample.buffer().expect("Could not get sample buffer");
                    let data = buffer
                        .map_readable()
                        .expect("Could not get map")
                        .as_slice()
                        .to_vec();
                    if let Err(_) = sender.send(data).await {
                        break;
                    }
                }
            }
            pipeline
                .iterate_pads()
                .foreach(|f| f.stop_task().expect("Could not stop task"))
                .expect("Could not iterate pads");
            pipeline
                .iterate_elements()
                .foreach(|f| {
                    f.set_state(gstreamer::State::Null)
                        .expect("Could not close element");
                    drop(f);
                })
                .expect("Could not iterate");
            pipeline
                .set_state(gstreamer::State::Null)
                .expect("tried to set state to null");
            drop(pipeline)
        });
        obj
    }

    pub fn width(&self) -> i32 {
        self.imp().width.get()
    }

    pub fn height(&self) -> i32 {
        self.imp().height.get()
    }

    /// Create a pipeline for a given video and return the pipeline and output (the appsink)
    fn create_pipeline(path: &str) -> (Pipeline, AppSink) {
        let pipeline = Pipeline::new();
        let source = ElementFactory::make("filesrc")
            .property("location", path)
            .build()
            .expect("Couldn't find file");
        let decodebin = ElementFactory::make("decodebin")
            .build()
            .expect("Could not create decodebin");
        let videoconvert = ElementFactory::make("videoconvert")
            .build()
            .expect("Could not create videoconvert");
        let videoscale = ElementFactory::make("videoscale")
            .build()
            .expect("Could not create videoscale");
        let caps = Caps::builder("video/x-raw").field("format", &"RGB").build();
        let appsink = ElementFactory::make("appsink")
            .property("caps", caps)
            .property("emit_signals", &true)
            .property("max_buffers", &1u32)
            .property("drop", &true)
            .build()
            .expect("Could not create imagesink");
        pipeline
            .add_many([&source, &decodebin, &videoconvert, &videoscale, &appsink])
            .expect("could not add elements to pipeline");
        Element::link_many([&source, &decodebin]).expect("Could not link 1");
        Element::link_many([&videoconvert, &videoscale, &appsink]).expect("Could not link 2");

        let videoconvert_weak = gstreamer::prelude::ObjectExt::downgrade(&videoconvert);
        decodebin.connect_pad_added(move |_, src| {
            if let Some(videoconvert) = videoconvert_weak.upgrade() {
                let sink_pad = videoconvert
                    .static_pad("sink")
                    .expect("Could not create sink");
                if !sink_pad.is_linked() {
                    src.link(&sink_pad).expect("Could not link pads");
                }
            }
        });

        let appsink = appsink
            .dynamic_cast::<gstreamer_app::AppSink>()
            .expect("Could not create appsink");
        (pipeline, appsink)
    }

    /// Connect to the frame which is data Vec<u8>
    pub fn connect_frame<F: Fn(Vec<u8>) + 'static>(
        receiver: async_channel::Receiver<Vec<u8>>,
        f: F,
    ) {
        spawn_future_local(async move {
            while let Ok(frame) = receiver.recv().await {
                f(frame);
            }
            println!("stop listening")
        });
    }
}
