use gdk4::builders::RGBABuilder;
use gdk4::{Monitor, Snapshot, Texture};
use gtk::graphene::{Point, Rect, Size};
use gtk::prelude::*;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{gio, glib, MediaFile};
use snafu::{OptionExt, Report};

use crate::errors::{DragonDisplayError, OtherSnafu};
use crate::try_emit;
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
    use std::cell::{Cell, OnceCell, RefCell};
    use std::sync::OnceLock;

    use gdk4::{Monitor, Texture};
    use glib::subclass::InitializingObject;
    use gtk::glib::subclass::Signal;
    use gtk::subclass::prelude::*;
    use gtk::{glib, Button, CompositeTemplate, MediaFile};
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
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for DdDisplayWindow {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "DdDisplayWindow";
        type Type = super::DdDisplayWindow;
        type ParentType = gtk::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Button::ensure_type();

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
        object
    }

    pub fn rotate_90(&self) {
        self.imp().rotation.replace_with(|rotation| match rotation {
            Rotation::None => Rotation::Clockwise,
            Rotation::Clockwise => Rotation::UpsideDown,
            Rotation::UpsideDown => Rotation::Counterclockwise,
            Rotation::Counterclockwise => Rotation::None,
        });
        self.apply_rotation();
    }

    pub fn rotate_180(&self) {
        self.imp().rotation.replace_with(|rotation| match rotation {
            Rotation::None => Rotation::UpsideDown,
            Rotation::Clockwise => Rotation::Counterclockwise,
            Rotation::UpsideDown => Rotation::None,
            Rotation::Counterclockwise => Rotation::Clockwise,
        });
        self.apply_rotation();
    }

    pub fn rotate_270(&self) {
        self.imp().rotation.replace_with(|rotation| match rotation {
            Rotation::None => Rotation::Counterclockwise,
            Rotation::Clockwise => Rotation::None,
            Rotation::UpsideDown => Rotation::Clockwise,
            Rotation::Counterclockwise => Rotation::UpsideDown,
        });
        self.apply_rotation();
    }

    /// Set the content of the display window to an image
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

        // apply rotation
        self.apply_rotation();
    }

    /// Set the content of the display window to a video
    pub fn set_video(&self, path_to_video: String) {
        self.disconnect_media();
        self.imp().texture.replace(None);
        let media_file = self
            .imp()
            .media_file
            .get()
            .expect("Expected media file to be set");
        media_file.set_filename(Some(&path_to_video));
        media_file.play();
        media_file.set_loop(true);
        media_file.set_muted(true);
        self.imp().content.set_paintable(Some(media_file));
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

    pub fn toggle_grid(&self) {
        if self.imp().grid.get() {
            self.imp().grid.replace(false);
            self.apply_rotation();
        } else {
            self.imp().grid.replace(true);
            self.apply_grid();
        }
    }

    /// Calculates and applies a grid that needs to be drawn over the current texture
    fn apply_grid(&self) {
        let binding = &*self.imp().texture.borrow();
        let texture = match binding {
            Some(t) => t,
            None => {
                return;
            }
        };
        let monitor = self.imp().monitor.get().expect("Monitor should be set");
        let real_width = monitor.width_mm();
        let real_height = monitor.height_mm();

        // TODO: dynamic grid color
        let black = RGBABuilder::new().red(0.0).green(0.0).blue(0.0).build();

        // amount of squares
        let height_amount = real_height / 25; // 25mm is about 1 inch
        let width_amount = real_width / 25;
        println!(
            "The amount of horizontal sections should be: {}",
            height_amount
        );

        // height and width of texture
        let width = texture.width() as f32;
        let height = texture.height() as f32;
        println!("the height of the texture is: {}", height);

        // the height and width of one square in pixels
        let height_square = height / height_amount as f32;
        let width_square = width / width_amount as f32;
        println!("Height per square: {}", height_square);

        let snapshot = gtk::Snapshot::new();
        snapshot.save();
        snapshot.append_texture(texture, &Rect::new(0.0, 0.0, width, height));

        let mut line_height = 0.0;
        while line_height < height {
            snapshot.append_color(&black, &Rect::new(0.0, line_height, width, 0.5));
            line_height += height_square;
        }
        snapshot.restore();
        let gridded_texture = match snapshot.to_paintable(Some(&Size::new(width, height))) {
            Some(t) => t,
            None => {
                println!("Could not create texture");
                return;
            }
        };
        self.imp().content.set_paintable(Some(&gridded_texture));
    }

    /// Applies a rotation to the currently stored texture and updates the current picture that is
    /// presented with the rotation
    fn apply_rotation(&self) {
        let binding = &*self.imp().texture.borrow();
        let texture = match binding {
            Some(t) => t,
            None => {
                return;
            }
        };

        let angle_degree = self.imp().rotation.borrow().get_angle_degree();
        let width = texture.width() as f32;
        let height = texture.height() as f32;
        let (new_width, new_height) = match angle_degree {
            90 | 270 => (height, width),
            _ => (width, height),
        };

        let snapshot = gtk::Snapshot::new();
        snapshot.save();
        snapshot.translate(&Point::new(new_width / 2.0, new_height / 2.0));
        snapshot.rotate(angle_degree as f32);
        snapshot.translate(&Point::new(-width / 2.0, -height / 2.0));
        snapshot.append_texture(texture, &Rect::new(0.0, 0.0, width, height));
        snapshot.restore();
        let rotated_texture = match snapshot.to_paintable(Some(&Size::new(new_width, new_height))) {
            Some(t) => t,
            None => {
                println!("Could not create texture");
                return;
            }
        };
        self.imp().content.set_paintable(Some(&rotated_texture));
    }

    /// Clear the media file and keep it alive to make sure it is cleared
    fn disconnect_media(&self) {
        if let Some(media) = self.imp().media_file.get() {
            media.set_playing(false);
            media.set_loop(false);
            media.set_filename(None::<String>);
            media.clear();
        }
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
