use gdk4::builders::RGBABuilder;
use gdk4::{Monitor, Texture, RGBA};
use gtk::gdk_pixbuf::Pixbuf;
use gtk::glib::{clone, Bytes};
use gtk::graphene::{Point, Rect, Size};
use gtk::prelude::*;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{gio, glib, MediaFile};
use snafu::{OptionExt, Report, ResultExt};
use vtt_rust::open_vtt;

use crate::errors::{DragonDisplayError, GlibSnafu, OtherSnafu};
use crate::videopipeline::VideoPipeline;
use crate::{try_emit, APP_ID};

use super::options::ColorPreset;
pub enum Rotation {
    None,
    Clockwise,
    UpsideDown,
    Counterclockwise,
}

impl Rotation {
    fn get_angle_degree(&self) -> i32 {
        match self {
            Rotation::None => 0,
            Rotation::Clockwise => 90,
            Rotation::UpsideDown => 180,
            Rotation::Counterclockwise => 270,
        }
    }
}

impl Default for Rotation {
    fn default() -> Self {
        Rotation::None
    }
}

mod imp {

    use crate::ui::display_window::Rotation;
    use crate::videopipeline::VideoPipeline;
    use std::cell::{Cell, OnceCell, RefCell};
    use std::sync::OnceLock;

    use gdk4::{Monitor, Texture, RGBA};
    use glib::subclass::InitializingObject;
    use gtk::glib::subclass::Signal;
    use gtk::subclass::prelude::*;
    use gtk::{glib, CompositeTemplate, MediaFile};
    use gtk::{prelude::*, Picture};

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dragon/display/display_window.ui")]
    pub struct DdDisplayWindow {
        #[template_child]
        pub content: TemplateChild<Picture>,
        pub fit: Cell<bool>,
        pub grid: Cell<bool>,
        pub rotation: RefCell<Rotation>,
        pub texture: RefCell<Option<Texture>>,
        pub media_file: OnceCell<MediaFile>,
        pub monitor: OnceCell<Monitor>,
        pub color: RefCell<Option<RGBA>>,
        pub gridline_width: Cell<f32>,
        pub pipeline: RefCell<Option<VideoPipeline>>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for DdDisplayWindow {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "DdDisplayWindow";
        type Type = super::DdDisplayWindow;
        type ParentType = gtk::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for DdDisplayWindow {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![Signal::builder("error")
                    .param_types([String::static_type(), bool::static_type()])
                    .build()]
            })
        }

        fn constructed(&self) {
            // Call "constructed" on parent
            self.parent_constructed();
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for DdDisplayWindow {}

    // Trait shared by all windows
    impl WindowImpl for DdDisplayWindow {}

    // Trait shared by all application windows
    impl ApplicationWindowImpl for DdDisplayWindow {}
}

glib::wrapper! {
    pub struct DdDisplayWindow(ObjectSubclass<imp::DdDisplayWindow>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
            @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                        gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl DdDisplayWindow {
    pub fn new(monitor: &Monitor) -> Self {
        let object = glib::Object::new::<Self>();
        object
            .imp()
            .media_file
            .set(MediaFile::new())
            .expect("Expected media file to not be set");
        object
            .imp()
            .monitor
            .set(monitor.to_owned())
            .expect("Expected monitor to not be set");
        let settings = gtk::gio::Settings::new(APP_ID);
        let color_index = settings.int("grid-color-preset") as u32;
        let color = ColorPreset::from_index(color_index).to_rgba();
        object.imp().color.replace(Some(color));

        let gridline_width = settings.double("grid-line-width") as f32;
        object.imp().gridline_width.set(gridline_width);
        let video_pipeline = VideoPipeline::new();
        object.imp().pipeline.replace(Some(video_pipeline));

        object
    }

    /// Disconnect the media that the display is holding on to and set the content to nothing
    pub fn reset(&self) {
        self.disconnect_media();
        self.imp().texture.replace(None);
        self.imp().content.set_paintable(None::<&Texture>);
    }

    /// Tries to rotate the current texture by 90 degrees, if there is no current texture the
    /// internal rotation value is still updated
    pub fn rotate_90(&self) {
        self.imp().rotation.replace_with(|rotation| match rotation {
            Rotation::None => Rotation::Clockwise,
            Rotation::Clockwise => Rotation::UpsideDown,
            Rotation::UpsideDown => Rotation::Counterclockwise,
            Rotation::Counterclockwise => Rotation::None,
        });
        self.redraw();
    }

    /// Tries to rotate the current texture by 180 degrees, if there is no current texture the
    /// internal rotation value is still updated
    pub fn rotate_180(&self) {
        self.imp().rotation.replace_with(|rotation| match rotation {
            Rotation::None => Rotation::UpsideDown,
            Rotation::Clockwise => Rotation::Counterclockwise,
            Rotation::UpsideDown => Rotation::None,
            Rotation::Counterclockwise => Rotation::Clockwise,
        });
        self.redraw();
    }

    /// Tries to rotate the current texture by 270 degrees, if there is no current texture the
    /// internal rotation value is still updated
    pub fn rotate_270(&self) {
        self.imp().rotation.replace_with(|rotation| match rotation {
            Rotation::None => Rotation::Counterclockwise,
            Rotation::Clockwise => Rotation::None,
            Rotation::UpsideDown => Rotation::Clockwise,
            Rotation::Counterclockwise => Rotation::UpsideDown,
        });
        self.redraw();
    }

    /// Update the texture of the display window and set it to an image that is at the given path
    pub fn set_image(&self, path_to_image: String) {
        self.disconnect_media();
        let texture = try_emit!(
            self,
            Texture::from_filename(&path_to_image)
                .ok()
                .context(OtherSnafu {
                    msg: format!("Could not load image at {}", &path_to_image)
                }),
            false
        );
        self.imp().texture.replace(Some(texture));

        // set the fit
        if self.imp().fit.get() {
            self.imp().content.set_content_fit(gtk::ContentFit::Fill);
        } else {
            self.imp().content.set_content_fit(gtk::ContentFit::Contain);
        }

        self.redraw();
    }

    /// Set the content of the display window to a video
    pub fn set_video(&self, path_to_video: String) {
        let (sender, receiver) = async_channel::unbounded();
        self.disconnect_media();

        
        let mut borrow = self.imp().pipeline.borrow_mut();
        let pipeline = borrow.as_mut().expect("No pipeline found");

        let (width, height) = pipeline.play_video(&path_to_video, sender);
        let stride = width*3;
        pipeline.connect_frame(receiver, 
            clone!(@weak self as obj => move |frame| {
                let pixbuf = Pixbuf::from_mut_slice(frame, gtk::gdk_pixbuf::Colorspace::Rgb, false, 8, width, height, stride);
                obj.imp().content.set_pixbuf(Some(&pixbuf));
            }),
        );
    }

    /// Set the vtt file and fog of war
    pub fn set_vtt(&self, path_to_vtt: String, fog_of_war: Vec<Rect>) {
        self.disconnect_media();
        let mut vtt = try_emit!(self, open_vtt(&path_to_vtt).ok().context(OtherSnafu {msg: "Failed to open vtt".to_string()}), false);
        let image = try_emit!(self, vtt.take_image().ok().context(OtherSnafu {msg: "Failed to get image from vtt file".to_string()}), false);
        let bytes = Bytes::from(&image);
        let texture = try_emit!(self, Texture::from_bytes(&bytes).context(GlibSnafu {msg: "Failed to create texture from image in vtt".to_string()}), false);
        self.imp().texture.replace(Some(texture));
        self.redraw_vtt(fog_of_war);
    }

    /// Toggle the content fit of the image, if there is no picture it will update the value but
    /// silently fail to update the picture
    pub fn toggle_fit(&self) {
        if self.imp().fit.get() {
            self.imp().fit.replace(false);
            self.imp().content.set_content_fit(gtk::ContentFit::Contain);
        } else {
            self.imp().fit.replace(true);
            self.imp().content.set_content_fit(gtk::ContentFit::Fill);
        }
    }

    /// Toggle a grid over the current texture, if there is no active texture it will update the
    /// value but silently fail to update the content.
    pub fn toggle_grid(&self) {
        if self.imp().grid.get() {
            self.imp().grid.replace(false);
        } else {
            self.imp().grid.replace(true);
        }
        self.redraw();
    }

    /// Update the color of the grid and redraw the texture
    pub fn update_grid_color(&self, color: RGBA) {
        self.imp().color.replace(Some(color));
        self.redraw();
    }

    /// Update the width of the grid lines
    pub fn set_gridline_width(&self, width: f32) {
        self.imp().gridline_width.replace(width);
        self.redraw();
    }

    /// Draws a grid in the given snapshot. This function needs the width and height of the current
    /// texture. This width and height should be updated to fit the rotation of the texture. It
    /// also needs the monitor that the image is displayed on to calculate the sizes of the
    /// squares.
    fn draw_grid(
        snapshot: &gtk::Snapshot,
        width: f32,
        height: f32,
        line_width: f32,
        color: &RGBA,
        monitor: &Monitor,
    ) {
        let total_vertical_squares = monitor.height_mm() / 25; // 25mm is about 1 inch
        let total_horizontal_squares = monitor.width_mm() / 25;

        // 10 squares, 1080p, image: 2200p 2200/10 = 220
        let pix_per_horizontal_square = width / total_horizontal_squares as f32;
        let pix_per_vertical_square = height / total_vertical_squares as f32;

        snapshot.save();
        let mut line_position = 0.0;
        while line_position < height as f32 {
            snapshot.append_color(
                color,
                &Rect::new(0.0, line_position - (line_width / 2.0), width, line_width),
            );
            line_position += pix_per_vertical_square as f32;
        }
        line_position = 0.0;
        while line_position < width as f32 {
            snapshot.append_color(
                color,
                &Rect::new(line_position - (line_width / 2.0), 0.0, line_width, height),
            );
            line_position += pix_per_horizontal_square as f32;
        }
        snapshot.restore();
    }

    /// Applies the given rotation to the snapshot needs a width and height of the texture to be
    /// rotated
    fn draw_rotation(snapshot: &gtk::Snapshot, width: f32, height: f32, rotation: &Rotation) {
        let angle_degree = rotation.get_angle_degree();
        let (new_width, new_height) = match angle_degree {
            90 | 270 => (height, width),
            _ => (width, height),
        };
        snapshot.translate(&Point::new(new_width / 2.0, new_height / 2.0));
        snapshot.rotate(angle_degree as f32);
        snapshot.translate(&Point::new(-width / 2.0, -height / 2.0));
    }

    /// Draws the given texture to the snapshot
    fn draw_texture(snapshot: &gtk::Snapshot, texture: &Texture) {
        let width = texture.width() as f32;
        let height = texture.height() as f32;
        snapshot.save();
        snapshot.append_texture(texture, &Rect::new(0.0, 0.0, width, height));
        snapshot.restore();
    }

    fn draw_fogofwar(snapshot: &gtk::Snapshot, fog_of_war: Vec<Rect>) {
        snapshot.save();
        let black = RGBABuilder::new()
            .red(0.)
            .blue(0.)
            .green(0.)
            .alpha(1.)
            .build();
        for rect in fog_of_war {
            snapshot.append_color(&black, &rect);
        }
        snapshot.restore();
    }

    /// Function called when the image needs to be redrawn. Creates a new snapshot, sets it up
    /// according to all the current settings and sets the content to the current texture.
    fn redraw(&self) {
        let binding = &*self.imp().texture.borrow();
        let texture = match binding {
            Some(t) => t,
            None => {
                return;
            }
        };

        let width = texture.width() as f32;
        let height = texture.height() as f32;
        let rotation = &self.imp().rotation.borrow();
        let (new_width, new_height) = match rotation.get_angle_degree() {
            90 | 270 => (height, width),
            _ => (width, height),
        };

        let snapshot = gtk::Snapshot::new();
    
        Self::draw_rotation(&snapshot, width, height, rotation);

        Self::draw_texture(&snapshot, texture);

        if self.imp().grid.get() {
            let monitor = self
                .imp()
                .monitor
                .get()
                .expect("Expected a monitor to be set");
            let color = &self.imp().color.borrow().expect("Expected color to be set");
            let line_width = self.imp().gridline_width.get();
            Self::draw_grid(&snapshot, new_width, new_height, line_width, color, monitor);
        }

        let paintable = match snapshot.to_paintable(Some(&Size::new(new_width, new_height))) {
            Some(t) => t,
            None => {
                return;
            }
        };
        self.imp().content.set_paintable(Some(&paintable));
    }

    fn redraw_vtt(&self, fog_of_war: Vec<Rect>) {
        let binding = &*self.imp().texture.borrow();
        let texture = match binding {
            Some(t) => t,
            None => {
                return;
            }
        };
        let snapshot = gtk::Snapshot::new();

        Self::draw_texture(&snapshot, texture); 

        Self::draw_fogofwar(&snapshot, fog_of_war);
        let width = texture.width() as f32;
        let height = texture.height() as f32;
        let paintable = match snapshot.to_paintable(Some(&Size::new(width, height))) {
            Some(t) => t,
            None => {
                return;
            }
        };
        self.imp().content.set_paintable(Some(&paintable));
    }

    /// Clear the media file and keep it alive to make sure it is cleared
    fn disconnect_media(&self) {
        let borrow = self.imp().pipeline.borrow_mut();
        let pipeline = borrow.as_ref().expect("Expected a pipeline");
        pipeline.stop_video();
    }

    /// Emit an error message based on the input error
    pub fn emit_error(&self, err: DragonDisplayError, fatal: bool) {
        let msg = Report::from_error(err).to_string();
        self.emit_by_name::<()>("error", &[&msg, &fatal]);
    }

    /// Signal emitted when an error occurs
    pub fn connect_error<F: Fn(&Self, String, bool) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            "error",
            true,
            glib::closure_local!(|window, msg, fatal| {
                f(window, msg, fatal);
            }),
        )
    }
}
