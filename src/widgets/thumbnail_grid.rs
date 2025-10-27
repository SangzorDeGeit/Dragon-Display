use std::path::PathBuf;

use crate::APP_ID;
use gtk::glib::clone;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{glib, Grid};
use gtk::{prelude::*, ToggleButton};

use super::thumbnail::{DdThumbnail, MediaType};

mod imp {
    use gtk::glib::subclass::Signal;
    use std::cell::{Cell, RefCell};
    use std::sync::OnceLock;

    use glib::subclass::InitializingObject;
    use gtk::subclass::prelude::*;
    use gtk::{glib, template_callbacks, Box, CompositeTemplate, Grid, ToggleButton};
    use gtk::{prelude::*, Button};

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dragon/display/thumbnail_grid.ui")]
    pub struct DdThumbnailGrid {
        #[template_child]
        pub main_box: TemplateChild<Box>,
        #[template_child]
        pub navigation_box: TemplateChild<Box>,
        #[template_child]
        pub next: TemplateChild<Button>,
        #[template_child]
        pub previous: TemplateChild<Button>,

        pub togglebuttons: RefCell<Vec<ToggleButton>>,
        pub current_grid_nr: Cell<usize>,
        pub page_vec: RefCell<Vec<Grid>>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for DdThumbnailGrid {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "DdThumbnailGrid";
        type ParentType = gtk::Widget;
        type Type = super::DdThumbnailGrid;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_layout_manager_type::<gtk::BoxLayout>();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[template_callbacks]
    impl DdThumbnailGrid {
        #[template_callback]
        fn handle_previous(&self, button: Button) {
            let page_vec = self.page_vec.borrow();
            let current_nr = self.current_grid_nr.get();
            let previous_nr = current_nr
                .checked_sub(1)
                .expect("Previous button should not have been able to be clicked");
            let current_page = page_vec.get(current_nr).expect("No current page was found");
            let previous_page = page_vec
                .get(previous_nr)
                .expect("No previous page was found");
            self.main_box.remove(current_page);
            self.main_box.prepend(previous_page);
            self.current_grid_nr.replace(previous_nr);
            if let None = previous_nr.checked_sub(1) {
                button.set_sensitive(false);
            }
            self.next.set_sensitive(true);
        }

        #[template_callback]
        fn handle_next(&self, button: Button) {
            let page_vec = self.page_vec.borrow();
            let current_nr = self.current_grid_nr.get();
            let mut next_nr = current_nr.wrapping_add(1);
            let current_page = page_vec.get(current_nr).expect("No current page was found");
            let next_page = page_vec.get(next_nr).expect("No next page was found");
            self.main_box.remove(current_page);
            self.main_box.prepend(next_page);
            self.current_grid_nr.replace(next_nr);

            next_nr = next_nr.wrapping_add(1);
            if let None = page_vec.get(next_nr) {
                button.set_sensitive(false);
            }
            self.previous.set_sensitive(true);
        }
    }
    // Trait shared by all GObjects
    impl ObjectImpl for DdThumbnailGrid {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![Signal::builder("path")
                    .param_types([String::static_type()])
                    .build()]
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
    impl WidgetImpl for DdThumbnailGrid {}
}

glib::wrapper! {
    pub struct DdThumbnailGrid(ObjectSubclass<imp::DdThumbnailGrid>)
        @extends gtk::Widget,
            @implements gtk::Actionable, gtk::Accessible, gtk::Buildable,
                        gtk::ConstraintTarget;
}

impl DdThumbnailGrid {
    /// Create a new image grid
    pub fn new(files: Vec<PathBuf>, t: &MediaType) -> Self {
        let object = glib::Object::new::<Self>();
        let mut prev_button: Option<ToggleButton> = None;
        for file in files {
            let thumbnail_image = DdThumbnail::new(&file, prev_button.as_ref(), t);
            thumbnail_image.connect_clicked(clone!(@weak object => move |button| {
                object.emit_by_name::<()>("path", &[&button.file()])
            }));
            object
                .imp()
                .togglebuttons
                .borrow_mut()
                .push(thumbnail_image.clone().into());
            prev_button = Some(thumbnail_image.upcast::<ToggleButton>())
        }
        object.populate_grids();
        object
    }

    /// Given a vector of paths to media files this function updates the buttons in the current
    /// grid.
    pub fn update(&self, images: Vec<PathBuf>, t: &MediaType) {
        // Figure out which buttons can be kept, which need to be upated and how many new
        let image_strings: Vec<String> = images
            .iter()
            .filter_map(|i| i.to_str())
            .map(|i| i.to_string())
            .collect();
        let (keep, replace): (Vec<DdThumbnail>, Vec<DdThumbnail>) = self
            .imp()
            .togglebuttons
            .take()
            .into_iter()
            .filter_map(|b| b.downcast::<DdThumbnail>().ok())
            .partition(|b| image_strings.contains(&b.file()));
        let keep_paths: Vec<String> = keep.iter().map(|b| b.file()).collect();
        let new_images: Vec<PathBuf> = images
            .into_iter()
            .filter(|i| !keep_paths.contains(&i.to_str().expect("failed conversion").to_string()))
            .collect();
        let mut keep: Vec<ToggleButton> = keep
            .into_iter()
            .map(|k| k.upcast::<ToggleButton>())
            .collect();

        // no new images need to be added
        if new_images.len() == 0 {
            self.imp().togglebuttons.replace(keep);
            self.populate_grids();
            return;
        }
        // new images need to be added
        // first go through all buttons that can be replaced
        let mut i = 0;
        for button in replace {
            if let Some(new_path) = new_images.get(i) {
                button.update(new_path);
                keep.push(button.upcast::<ToggleButton>());
                i += 1;
            }
        }
        // if there are still new images needed create them
        while let Some(new_path) = new_images.get(i) {
            let prev_button = keep.last();
            let thumbnail = DdThumbnail::new(new_path, prev_button, t);
            thumbnail.connect_clicked(clone!(@weak self as obj => move |button| {
                obj.emit_by_name::<()>("path", &[&button.file()])
            }));
            keep.push(thumbnail.upcast::<ToggleButton>());
            i += 1;
        }
        self.imp().togglebuttons.replace(keep);
        self.populate_grids();
    }

    /// Create an amount of grids for the given amount of storage needed and populate these grids
    /// with the self.togglebuttons
    pub fn populate_grids(&self) {
        if let Some(child) = self.imp().main_box.first_child() {
            if let Some(grid) = child.downcast_ref::<Grid>() {
                self.imp().main_box.remove(grid);
            }
        }

        let settings = gtk::gio::Settings::new(APP_ID);
        let mut column = settings.int("imagegrid-column-amount");
        let mut row = settings.int("imagegrid-row-amount");
        if column <= 0 {
            column = 3;
        }
        if row <= 0 {
            row = 3;
        }
        let total_files = self.imp().togglebuttons.borrow().len() as f64;
        let files_per_page = (row * column) as f64;
        let grids_needed = (total_files / files_per_page).ceil() as usize;
        let mut new_grids = Vec::new();
        for _ in 0..grids_needed {
            let grid = Grid::builder()
                .halign(gtk::Align::Fill)
                .valign(gtk::Align::Fill)
                .hexpand(true)
                .vexpand(true)
                .row_spacing(3)
                .column_spacing(3)
                .build();
            new_grids.push(grid);
        }
        self.imp().page_vec.replace(Vec::new());
        let mut i = 0;
        for togglebutton in self.imp().togglebuttons.borrow().iter() {
            let grid_nr = i / files_per_page as i32;
            let grid = new_grids.get(grid_nr as usize).expect("Expected a grid");
            grid.attach(togglebutton, i % column, (i / column) % row, 1, 1);
            i += 1;
        }
        if new_grids.len() <= 1 {
            self.imp().navigation_box.set_child_visible(false);
        }
        self.imp().previous.set_sensitive(false);
        self.imp().current_grid_nr.replace(0);
        self.imp()
            .main_box
            .prepend(new_grids.get(0).expect("Expected a grid"));
        self.imp().page_vec.replace(new_grids);
    }

    /// Signal emitted when an image is clicked
    pub fn connect_path<F: Fn(&Self, String) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "path",
            true,
            glib::closure_local!(|window, path| {
                f(window, path);
            }),
        )
    }
}
