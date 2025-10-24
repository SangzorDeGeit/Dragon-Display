use std::cell::Cell;
use std::fs::read_dir;
use std::path::PathBuf;
use std::rc::Rc;

use gtk::glib::clone;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{gio, glib};
use gtk::{prelude::*, Label};
use snafu::Report;
use snafu::ResultExt;

use crate::config::{IMAGE_EXTENSIONS, VIDEO_EXTENSIONS, VTT_EXTENSIONS};
use crate::errors::*;
use crate::widgets::thumbnail::MediaType;
use crate::widgets::thumbnail_grid::DdThumbnailGrid;
use crate::widgets::vtt_area::DdVttArea;

mod imp {

    use std::cell::{Cell, OnceCell};
    use std::sync::OnceLock;

    use glib::subclass::InitializingObject;
    use gtk::glib::subclass::Signal;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::{glib, template_callbacks, Box, Button, CompositeTemplate, Stack, StackSwitcher};

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dragon/display/control_window.ui")]
    pub struct DdControlWindow {
        #[template_child]
        pub stack: TemplateChild<Stack>,
        #[template_child]
        pub stackswitcher: TemplateChild<StackSwitcher>,
        #[template_child]
        pub images: TemplateChild<Box>,
        #[template_child]
        pub videos: TemplateChild<Box>,
        #[template_child]
        pub vtts: TemplateChild<Box>,
        #[template_child]
        pub options_button: TemplateChild<Button>,
        pub campaign_path: OnceCell<String>,
        pub has_images: Cell<bool>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for DdControlWindow {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "DdControlWindow";
        type Type = super::DdControlWindow;
        type ParentType = gtk::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Button::ensure_type();

            klass.bind_template();
            klass.bind_template_callbacks()
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[template_callbacks]
    impl DdControlWindow {
        #[template_callback]
        fn handle_refresh(&self, _: Button) {
            self.obj().emit_by_name::<()>("refresh", &[]);
        }

        #[template_callback]
        fn handle_options(&self, _: Button) {
            self.obj().emit_by_name::<()>("options", &[]);
        }

        #[template_callback]
        fn handle_reset_display(&self, _: Button) {
            self.obj().emit_by_name::<()>("reset-display", &[]);
        }

        #[template_callback]
        fn handle_rotate90(&self, _: Button) {
            self.obj().emit_by_name::<()>("rotate90", &[]);
        }

        #[template_callback]
        fn handle_rotate180(&self, _: Button) {
            self.obj().emit_by_name::<()>("rotate180", &[]);
        }

        #[template_callback]
        fn handle_rotate270(&self, _: Button) {
            self.obj().emit_by_name::<()>("rotate270", &[]);
        }

        #[template_callback]
        fn handle_fit(&self, _: Button) {
            self.obj().emit_by_name::<()>("fit", &[]);
        }

        #[template_callback]
        fn handle_grid(&self, _: Button) {
            self.obj().emit_by_name::<()>("grid", &[]);
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for DdControlWindow {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder("refresh").build(),
                    Signal::builder("options").build(),
                    Signal::builder("reset-display").build(),
                    Signal::builder("rotate90").build(),
                    Signal::builder("rotate180").build(),
                    Signal::builder("rotate270").build(),
                    Signal::builder("image")
                        .param_types([String::static_type()])
                        .build(),
                    Signal::builder("video")
                        .param_types([String::static_type()])
                        .build(),
                    Signal::builder("fit").build(),
                    Signal::builder("grid").build(),
                    Signal::builder("error")
                        .param_types([String::static_type(), bool::static_type()])
                        .build(),
                ]
            })
        }

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
    impl WidgetImpl for DdControlWindow {}

    // Trait shared by all windows
    impl WindowImpl for DdControlWindow {}

    // Trait shared by all application windows
    impl ApplicationWindowImpl for DdControlWindow {}
}

glib::wrapper! {
    pub struct DdControlWindow(ObjectSubclass<imp::DdControlWindow>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
            @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                        gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl DdControlWindow {
    pub fn new(app: &adw::Application, campaign_path: String) -> Result<Self, DragonDisplayError> {
        // set all properties
        let object = glib::Object::new::<Self>();
        object.set_application(Some(app));
        object
            .imp()
            .campaign_path
            .set(campaign_path.clone())
            .expect("Expected campaign path to not be set");
        object
            .imp()
            .stackswitcher
            .set_stack(Some(&object.imp().stack));
        let (images, _vtts, videos) = object.seperate_media(campaign_path)?;
        if images.len() == 0 {
            let label = Label::builder()
                .label("You have no images")
                .vexpand(true)
                .hexpand(true)
                .valign(gtk::Align::Center)
                .halign(gtk::Align::Center)
                .build();
            object.imp().images.append(&label);
        } else {
            let image_grid = DdThumbnailGrid::new(images, &MediaType::Image);
            object.imp().images.append(&image_grid);
            image_grid.connect_path(clone!(@weak object => move |_, path|{
                object.emit_by_name::<()>("image", &[&path]);
            }));
        }
        // get all vtts from the folder
        // setup the vtt grid
        let vtt_area = DdVttArea::new();
        object.imp().vtts.append(&vtt_area);
        let pressed = Rc::new(Cell::new(0));

        vtt_area.connect_pressed(clone!(@strong pressed => move |_, n, x, y| {
            pressed.set(n);
        }));

        vtt_area.connect_stopped(clone!(@strong pressed => move |_| {
            if pressed.get() > 0 {
                println!("long press");
                pressed.set(pressed.get()+1);
            }
        }));
        vtt_area.connect_released(clone!(@strong pressed => move |_, n| {
            let old_n = pressed.get();
            if n == old_n {
                println!("short press");
            }
            pressed.set(0);
        }));

        if videos.len() == 0 {
            let label = Label::builder()
                .label("You have no videos")
                .vexpand(true)
                .hexpand(true)
                .valign(gtk::Align::Center)
                .halign(gtk::Align::Center)
                .build();
            object.imp().videos.append(&label);
        } else {
            let video_grid = DdThumbnailGrid::new(videos, &MediaType::Video);
            object.imp().videos.append(&video_grid);
            video_grid.connect_path(clone!(@weak object => move |_, path| {
                object.emit_by_name::<()>("video", &[&path]);
            }));
        }
        Ok(object)
    }

    /// Update the grid of thumbnails
    pub fn update(&self) -> Result<(), DragonDisplayError> {
        let campaign_path = self
            .imp()
            .campaign_path
            .get()
            .expect("Expected a campaign path")
            .clone();
        let (images, _vtts, videos) = self.seperate_media(campaign_path)?;
        if images.len() > 0 {
            let image_child = self.imp().images.first_child().expect("Expected a child");
            if let Some(grid) = image_child.downcast_ref::<DdThumbnailGrid>() {
                grid.update(images, &MediaType::Image);
            }
        } else {
            let child = self.imp().images.first_child().expect("Expected a child");
            self.imp().images.remove(&child);
            let label = Label::builder()
                .label("You have no images")
                .halign(gtk::Align::Center)
                .hexpand(true)
                .valign(gtk::Align::Center)
                .vexpand(true)
                .build();
            self.imp().images.append(&label);
        }
        // update vtts

        if videos.len() > 0 {
            let video_child = self.imp().videos.first_child().expect("Expected a child");
            if let Some(grid) = video_child.downcast_ref::<DdThumbnailGrid>() {
                grid.update(videos, &MediaType::Video);
            }
        } else {
            let child = self.imp().videos.first_child().expect("Expected a child");
            self.imp().videos.remove(&child);
            let label = Label::builder()
                .label("You have no videos")
                .halign(gtk::Align::Center)
                .hexpand(true)
                .valign(gtk::Align::Center)
                .vexpand(true)
                .build();
            self.imp().videos.append(&label);
        }

        Ok(())
    }

    /// Reads all files in the given folder and seperates images, vtt files and videos,
    /// returns three vectors of path variables (images, vtts, videos)
    fn seperate_media(
        &self,
        campaign_path: String,
    ) -> Result<(Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>), DragonDisplayError> {
        let files = read_dir(campaign_path).context(IOSnafu {
            msg: "Could not read the campaign directory".to_string(),
        })?;

        let files: Vec<PathBuf> = files
            .filter_map(|f| f.ok())
            .map(|f| f.path())
            .filter(|f| f.to_str().is_some() && f.extension().is_some())
            .filter(|f| f.extension().unwrap().to_str().is_some())
            .collect();

        let (images, other): (Vec<PathBuf>, Vec<PathBuf>) = files
            .into_iter()
            .partition(|f| IMAGE_EXTENSIONS.contains(&f.extension().unwrap().to_str().unwrap()));
        let (vtts, other): (Vec<PathBuf>, Vec<PathBuf>) = other
            .into_iter()
            .partition(|f| VTT_EXTENSIONS.contains(&f.extension().unwrap().to_str().unwrap()));
        let videos: Vec<PathBuf> = other
            .into_iter()
            .filter(|f| VIDEO_EXTENSIONS.contains(&f.extension().unwrap().to_str().unwrap()))
            .collect();

        Ok((images, vtts, videos))
    }

    /// Set the options button of the control panel to sensitive (true or false)
    pub fn set_options_sensitive(&self, sensitive: bool) {
        self.imp().options_button.set_sensitive(sensitive);
    }

    /**
     * ----------------------------------
     *
     * Signal connect functions
     *
     * --------------------------------
     **/

    /// Signal emitted when an refresh button is pressed
    pub fn connect_refresh<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "refresh",
            true,
            glib::closure_local!(|window| {
                f(window);
            }),
        )
    }

    /// Signal emitted when an options button is pressed
    pub fn connect_options<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "options",
            true,
            glib::closure_local!(|window| {
                f(window);
            }),
        )
    }

    /// Signal emitted when an options button is pressed
    pub fn connect_reset_display<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "reset-display",
            true,
            glib::closure_local!(|window| {
                f(window);
            }),
        )
    }

    /// Signal emitted when an rotate90 button is pressed
    pub fn connect_rotate90<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "rotate90",
            true,
            glib::closure_local!(|window| {
                f(window);
            }),
        )
    }

    /// Signal emitted when an rotate180 button is pressed
    pub fn connect_rotate180<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "rotate180",
            true,
            glib::closure_local!(|window| {
                f(window);
            }),
        )
    }

    /// Signal emitted when an rotate270 button is pressed
    pub fn connect_rotate270<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "rotate270",
            true,
            glib::closure_local!(|window| {
                f(window);
            }),
        )
    }

    /// Signal emitted when an fit to size button is pressed
    pub fn connect_fit<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "fit",
            true,
            glib::closure_local!(|window| {
                f(window);
            }),
        )
    }

    /// Signal emitted when the grid button is pressed
    pub fn connect_grid<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "grid",
            true,
            glib::closure_local!(|window| {
                f(window);
            }),
        )
    }

    /// Signal emitted image in the grid is clicked
    pub fn connect_image<F: Fn(&Self, String) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "image",
            true,
            glib::closure_local!(|window, path| {
                f(window, path);
            }),
        )
    }

    /// Signal emitted video in the grid is clicked
    pub fn connect_video<F: Fn(&Self, String) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "video",
            true,
            glib::closure_local!(|window, path| {
                f(window, path);
            }),
        )
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
