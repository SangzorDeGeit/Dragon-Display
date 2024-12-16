use std::path::PathBuf;

use crate::config::IMAGE_EXTENSIONS;
use crate::program_manager::ControlWindowMessage;
use crate::widgets::thumbnail_image::DdThumbnailImage;
use crate::APP_ID;
use async_channel::Sender;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{glib, Grid};
use gtk::{prelude::*, ToggleButton};

mod imp {
    use async_channel::Sender;
    use std::cell::{Cell, RefCell};

    use glib::subclass::InitializingObject;
    use gtk::subclass::prelude::*;
    use gtk::{glib, template_callbacks, Box, CompositeTemplate, Grid};
    use gtk::{prelude::*, Button};

    use crate::program_manager::ControlWindowMessage;

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
        pub current_grid_nr: Cell<usize>,
        pub page_vec: RefCell<Vec<Grid>>,
        pub sender: RefCell<Option<Sender<ControlWindowMessage>>>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for DdThumbnailGrid {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "DdThumbnailGrid";
        type ParentType = gtk::Widget;
        type Type = super::DdThumbnailGrid;

        fn class_init(klass: &mut Self::Class) {
            Button::ensure_type();

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
    pub fn new(sender: Sender<ControlWindowMessage>, files: Vec<PathBuf>) -> Self {
        // set all properties
        let object = glib::Object::new::<Self>();
        let imp = object.imp();
        imp.sender.replace(Some(sender));
        Self::initialize(imp, files);

        object
    }

    fn initialize(imp: &imp::DdThumbnailGrid, files: Vec<PathBuf>) {
        let sender = imp.sender.borrow().clone().expect("Sender not found");
        let settings = gtk::gio::Settings::new(APP_ID);
        let mut column = settings.int("imagegrid-column-amount");
        let mut row = settings.int("imagegrid-row-amount");
        if column <= 0 {
            column = 3;
        }
        if row <= 0 {
            row = 3;
        }
        let file_amount = files.len() as f64;
        let files_per_page = (row * column) as f64;
        // the amount of pages needed is the amount of files divided by the amount of files per
        // page rounded up
        let pages_needed = (file_amount / files_per_page).ceil() as i32;
        let mut page_vec = Vec::new();
        let mut prev_button: Option<&ToggleButton> = None;
        for _ in 0..pages_needed {
            let page = Grid::builder()
                .halign(gtk::Align::Fill)
                .valign(gtk::Align::Fill)
                .hexpand(true)
                .vexpand(true)
                .row_spacing(3)
                .column_spacing(3)
                .build();
            page_vec.push(page);
        }
        // If there are no pages (in other words there are no files to display) remove prev and
        // next and return
        if let None = page_vec.get(0) {
            imp.main_box.remove(
                &imp.main_box
                    .first_child()
                    .expect("Did not find navigation buttons"),
            );
            return;
        }
        let mut filling_page = 0;
        let mut i = 0;
        let mut thumbnail_image: DdThumbnailImage;
        for file in files {
            // the validity of th extension was already checked in control_window
            let extension = file
                .extension()
                .expect("Could not get file extension")
                .to_str()
                .expect("Could not convert to str");
            if IMAGE_EXTENSIONS.contains(&extension) {
                thumbnail_image = DdThumbnailImage::new(&file, sender.clone(), prev_button);
                let filling_grid = page_vec
                    .get(filling_page)
                    .expect("Not enough pages created");
                thumbnail_image.set_halign(gtk::Align::Fill);
                thumbnail_image.set_valign(gtk::Align::Fill);
                thumbnail_image.set_hexpand(true);
                thumbnail_image.set_vexpand(true);
                prev_button = Some(thumbnail_image.get_togglebutton());
                filling_grid.attach(&thumbnail_image, i % column, i / column, 1, 1);
            } else {
                continue;
            }
            i += 1;
            if i % (files_per_page as i32) == 0 {
                i = 0;
                filling_page += 1;
            }
        }
        // Set the first page as the displayed page
        imp.previous.set_sensitive(false);
        imp.current_grid_nr.replace(0);
        imp.main_box
            .prepend(page_vec.get(0).expect("Could not find page"));
        if page_vec.len() <= 1 {
            imp.main_box.remove(
                &imp.main_box
                    .last_child()
                    .expect("Did not find navigation box"),
            );
        }
        imp.page_vec.replace(page_vec);
    }
}
