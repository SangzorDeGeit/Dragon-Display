use gtk::glib::clone;
use gtk::glib::object::ObjectExt;
use gtk::prelude::WidgetExt;
use gtk::{glib, GestureClick};
use std::sync::OnceLock;

use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

mod imp {
    use gtk::{
        glib::{
            subclass::{InitializingObject, Signal},
            types::StaticType,
        },
        Picture,
    };

    use super::*;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dragon/display/vtt_area.ui")]
    pub struct DdVttArea {
        #[template_child]
        pub image: TemplateChild<Picture>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for DdVttArea {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "DdVttArea";
        type Type = super::DdVttArea;
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
    impl ObjectImpl for DdVttArea {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder("pressed")
                        .param_types([i32::static_type(), f64::static_type(), f64::static_type()])
                        .build(),
                    Signal::builder("stopped").build(),
                    Signal::builder("released")
                        .param_types([i32::static_type()])
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
    impl WidgetImpl for DdVttArea {}
}

glib::wrapper! {
    pub struct DdVttArea(ObjectSubclass<imp::DdVttArea>)
        @extends gtk::Widget,
            @implements gtk::Actionable, gtk::Accessible, gtk::Buildable,
                        gtk::ConstraintTarget;
}

impl DdVttArea {
    pub fn new() -> Self {
        let object = glib::Object::new::<Self>();
        object
            .imp()
            .image
            .set_filename(Some("test_gd/Maled_huis.jpg"));
        println!("width: {}", object.imp().image.width());
        println!("height: {}", object.imp().image.height());
        // say image width = 2000 pix
        // real image = 10x10 with 100 pix per grid = 1000 pix
        // if we click on (100, 100), in the real image we click on (50, 50)
        //
        // aspect ratio = real_image/image;
        // coordinate*ratio = real image coordinate

        let clickable_area = GestureClick::builder().button(0).build();
        clickable_area.connect_pressed(clone!(@weak object => move |_, n, x, y| {
            object.emit_by_name::<()>("pressed", &[&n, &x, &y]);
        }));

        clickable_area.connect_stopped(clone!(@weak object => move |_| {
            object.emit_by_name::<()>("stopped", &[]);
        }));

        clickable_area.connect_released(clone!(@weak object => move |_,n,_,_| {
            object.emit_by_name::<()>("released", &[&n]);
        }));

        object.add_controller(clickable_area);
        object
    }

    /// Signal emitted when an error occurs
    pub fn connect_pressed<F: Fn(&Self, i32, f64, f64) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            "pressed",
            true,
            glib::closure_local!(|area, n, x, y| {
                f(area, n, x, y);
            }),
        )
    }

    /// Signal emitted when an error occurs
    pub fn connect_stopped<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "stopped",
            true,
            glib::closure_local!(|area| {
                f(area);
            }),
        )
    }

    /// Signal emitted when an error occurs
    pub fn connect_released<F: Fn(&Self, i32) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "released",
            true,
            glib::closure_local!(|area, n| {
                f(area, n);
            }),
        )
    }
}
