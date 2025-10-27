use async_channel::Receiver;
use async_channel::Sender;
use gstreamer::glib::spawn_future_local;
use gstreamer::prelude::*;
use gstreamer::ClockTime;
use gstreamer::Pipeline;
use gstreamer::{Caps, Element, ElementFactory};
use gstreamer::{SeekFlags, SeekType};
use gstreamer_app::AppSink;
use gtk::gdk_pixbuf::Pixbuf;

use crate::runtime;

pub struct VideoPipeline {
    pipeline: Pipeline,
    appsink: AppSink,
    source: Element,
    sender: Option<Sender<Vec<u8>>>,
}

impl VideoPipeline {
    /// Create a new pipeline element, this element should be reused instead of creating new ones
    pub fn new() -> Self {
        gstreamer::init().expect("Could not initialize gstreamer");
        let pipeline = Pipeline::new();
        let source = ElementFactory::make("filesrc")
            .build()
            .expect("Could not make source");
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
        Self {
            pipeline,
            appsink,
            source,
            sender: None,
        }
    }

    /// Play the video  file and start sending frames through the sender, call
    /// connect_frames. This function returns the width and height of the video
    pub fn play_video(&mut self, path: &str, sender: Sender<Vec<u8>>) -> (i32, i32) {
        self.source.set_property("location", path);
        let appsink = self.appsink.clone();
        self.sender = Some(sender.clone());
        self.pipeline
            .set_state(gstreamer::State::Playing)
            .expect("Could not set state to playing");
        let sample = appsink.pull_preroll().expect("Could not pull first sample");
        let caps = sample.caps().expect("Could not get caps");
        let structure = caps.structure(0).expect("Could not get caps structure");
        let width = structure.get::<i32>("width").expect("Could not get width");
        let height = structure
            .get::<i32>("height")
            .expect("Could not get height");
        let pipeline = self.pipeline.clone();

        runtime().spawn(async move {
            loop {
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
                } else {
                    let seek_event = gstreamer::event::Seek::new(
                        1.0,
                        SeekFlags::FLUSH | SeekFlags::ACCURATE,
                        SeekType::Set,
                        ClockTime::ZERO,
                        SeekType::End,
                        ClockTime::NONE,
                    );
                    pipeline.send_event(seek_event);
                }
            }
        });
        (width, height)
    }

    /// Stop video playing closing the frame channel
    pub fn stop_video(&self) {
        self.pipeline
            .set_state(gstreamer::State::Ready)
            .expect("Could not set state to ready");
        if let Some(sender) = &self.sender {
            sender.close();
        }
    }

    pub fn thumbnail(&self, path: &str) -> Pixbuf {
        self.source.set_property("location", path);
        self.pipeline
            .set_state(gstreamer::State::Playing)
            .expect("Could not set state to playing");
        let sample = self.appsink.pull_preroll().expect("Could not get sample");
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
        self.pipeline
            .set_state(gstreamer::State::Ready)
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

    /// Connect to the sender that sends frames
    pub fn connect_frame<F: Fn(Vec<u8>) + 'static>(&self, receiver: Receiver<Vec<u8>>, f: F) {
        spawn_future_local(async move {
            while let Ok(frame) = receiver.recv().await {
                f(frame)
            }
        });
    }
}
