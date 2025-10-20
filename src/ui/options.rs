use std::cell::Cell;

use adw::Application;
use gdk4::builders::RGBABuilder;
use gdk4::RGBA;
use gtk::glib::clone;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{gio, glib};
use gtk::{prelude::*, Adjustment};

use crate::APP_ID;
pub const MAX_COLUMN_ROW_AMOUNT: f64 = 20.0;
pub const MIN_COLUMN_ROW_AMOUNT: f64 = 1.0;
pub const MIN_GRID_WIDTH: f64 = 0.1;
pub const MAX_GRID_WIDTH: f64 = 5.0;

/// To avoid errors the order of this list should be equal to the order of the dropdown list
/// defined in the options.ui
pub enum ColorPreset {
    Black,
    White,
    Red,
    Green,
    Blue,
}

impl ColorPreset {
    /// Create a new color preset based on the dropdown index
    pub fn from_index(index: u32) -> Self {
        match index {
            0 => Self::Black,
            1 => Self::White,
            2 => Self::Red,
            3 => Self::Green,
            4 => Self::Blue,
            _ => panic!("Found invalid index"),
        }
    }

    pub fn to_rgba(&self) -> RGBA {
        use ColorPreset as C;
        match self {
            C::Black => RGBABuilder::new().red(0.0).green(0.0).blue(0.0).build(),
            C::White => RGBABuilder::new()
                .red(255.0)
                .green(255.0)
                .blue(255.0)
                .build(),
            C::Red => RGBABuilder::new().red(255.0).green(0.0).blue(0.0).build(),
            C::Green => RGBABuilder::new().red(0.0).green(255.0).blue(0.0).build(),
            C::Blue => RGBABuilder::new().red(0.0).green(0.0).blue(255.0).build(),
        }
    }
}

mod imp {
    use std::sync::OnceLock;

    use glib::subclass::InitializingObject;
    use gtk::glib::subclass::Signal;
    use gtk::subclass::prelude::*;
    use gtk::{glib, Button, CompositeTemplate, DropDown, SpinButton};
    use gtk::{prelude::*, template_callbacks};

    use crate::APP_ID;

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dragon/display/options.ui")]
    pub struct DdOptionsWindow {
        #[template_child]
        pub row: TemplateChild<SpinButton>,
        #[template_child]
        pub column: TemplateChild<SpinButton>,
        #[template_child]
        pub color_dropdown: TemplateChild<DropDown>,
        #[template_child]
        pub gridline_width: TemplateChild<SpinButton>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for DdOptionsWindow {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "DdOptionsWindow";
        type Type = super::DdOptionsWindow;
        type ParentType = gtk::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Button::ensure_type();

            klass.bind_template();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[template_callbacks]
    impl DdOptionsWindow {
        #[template_callback]
        fn handle_confirm(&self, _: Button) {
            let settings = gtk::gio::Settings::new(APP_ID);
            settings
                .set_int("imagegrid-row-amount", self.row.value() as i32)
                .expect("Could not update row");
            settings
                .set_int("imagegrid-column-amount", self.column.value() as i32)
                .expect("Could not update column");
            settings
                .set_int("grid-color-preset", self.color_dropdown.selected() as i32)
                .expect("Could not color preset");
            settings
                .set_double("grid-line-width", self.gridline_width.value())
                .expect("Could not color preset");
            self.obj().emit_by_name::<()>("confirm", &[]);
        }

        #[template_callback]
        fn handle_default(&self, _: Button) {
            self.row.set_value(3.0);
            self.column.set_value(3.0);
            self.color_dropdown.set_selected(0);
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for DdOptionsWindow {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder("confirm").build(),
                    Signal::builder("color")
                        .param_types([u32::static_type()])
                        .build(),
                    Signal::builder("grid-line-width")
                        .param_types([f32::static_type()])
                        .build(),
                ]
            })
        }

        fn constructed(&self) {
            // Call "constructed" on parent
            self.parent_constructed();
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for DdOptionsWindow {}

    // Trait shared by all windows
    impl WindowImpl for DdOptionsWindow {}

    // Trait shared by all application windows
    impl ApplicationWindowImpl for DdOptionsWindow {}
}

glib::wrapper! {
    pub struct DdOptionsWindow(ObjectSubclass<imp::DdOptionsWindow>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
            @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                        gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl DdOptionsWindow {
    pub fn new(app: &Application) -> Self {
        // set all properties
        let object = glib::Object::new::<Self>();
        object.set_property("application", app);
        let imp = object.imp();
        imp.row
            .set_range(MIN_COLUMN_ROW_AMOUNT, MAX_COLUMN_ROW_AMOUNT);
        imp.column
            .set_range(MIN_COLUMN_ROW_AMOUNT, MAX_COLUMN_ROW_AMOUNT);
        let settings = gtk::gio::Settings::new(APP_ID);
        let columns = settings.int("imagegrid-column-amount") as f64;
        let rows = settings.int("imagegrid-row-amount") as f64;
        let row_adjustment = Adjustment::new(
            rows,
            MIN_COLUMN_ROW_AMOUNT,
            MAX_COLUMN_ROW_AMOUNT,
            1.0,
            10.0,
            0.0,
        );
        let column_adjustment = Adjustment::new(
            columns,
            MIN_COLUMN_ROW_AMOUNT,
            MAX_COLUMN_ROW_AMOUNT,
            1.0,
            10.0,
            0.0,
        );
        imp.row.set_adjustment(&row_adjustment);
        imp.column.set_adjustment(&column_adjustment);

        let current_color_preset = Cell::new(imp.color_dropdown.selected());
        let color_index = settings.int("grid-color-preset") as u32;
        imp.color_dropdown.set_selected(color_index);
        imp.color_dropdown
            .connect_selected_notify(clone!(@weak object => move |dropdown| {
                if current_color_preset.get() != dropdown.selected() {
                    current_color_preset.set(dropdown.selected());
                    object.emit_by_name::<()>("color", &[&dropdown.selected()])
                }
            }));

        let gridline_width = settings.double("grid-line-width");
        let gridline_adjustment = Adjustment::new(
            gridline_width,
            MIN_GRID_WIDTH,
            MAX_GRID_WIDTH,
            0.1,
            1.0,
            0.0,
        );
        imp.gridline_width.set_adjustment(&gridline_adjustment);
        gridline_adjustment.connect_value_changed(clone!(@weak object => move |adjustment| {
            object.emit_by_name::<()>("grid-line-width", &[&(adjustment.value() as f32)]);
        }));

        object
    }

    /// Signal emitted when the confirm button is pressed
    pub fn connect_confirm<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "confirm",
            true,
            glib::closure_local!(|window| {
                f(window);
            }),
        )
    }

    /// Signal emitted when the confirm button is pressed
    pub fn connect_grid_line_width<F: Fn(&Self, f32) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            "grid-line-width",
            true,
            glib::closure_local!(|window, width| {
                f(window, width);
            }),
        )
    }

    /// Signal emitted when a new color is selected
    pub fn connect_color<F: Fn(&Self, u32) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "color",
            true,
            glib::closure_local!(|window, color| {
                f(window, color);
            }),
        )
    }
}
