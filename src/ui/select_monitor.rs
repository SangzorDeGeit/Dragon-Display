use gdk4::{Display, Monitor};
use gtk::glib::clone;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{gio, glib};
use gtk::{prelude::*, Button};
use snafu::OptionExt;

use crate::errors::{DragonDisplayError, OtherSnafu};

mod imp {
    use std::sync::OnceLock;

    use gdk4::Monitor;
    use glib::subclass::InitializingObject;
    use gtk::glib::subclass::Signal;
    use gtk::prelude::StaticType;
    use gtk::subclass::prelude::*;
    use gtk::Grid;
    use gtk::{glib, CompositeTemplate};

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dragon/display/select_monitor.ui")]
    pub struct SelectMonitorWindow {
        #[template_child]
        pub monitor_grid: TemplateChild<Grid>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for SelectMonitorWindow {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "DdSelectMonitorWindow";
        type Type = super::SelectMonitorWindow;
        type ParentType = gtk::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for SelectMonitorWindow {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![Signal::builder("monitor")
                    .param_types([Monitor::static_type()])
                    .build()]
            })
        }
        fn constructed(&self) {
            // Call "constructed" on parent
            self.parent_constructed();
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for SelectMonitorWindow {}

    // Trait shared by all windows
    impl WindowImpl for SelectMonitorWindow {}

    // Trait shared by all application windows
    impl ApplicationWindowImpl for SelectMonitorWindow {}
}

glib::wrapper! {
    pub struct SelectMonitorWindow(ObjectSubclass<imp::SelectMonitorWindow>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
            @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                        gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl SelectMonitorWindow {
    pub fn new(app: &gtk::Application) -> Result<Self, DragonDisplayError> {
        // set all properties
        let object = glib::Object::new::<Self>();
        object.set_property("application", app);
        let display = Display::default().context(OtherSnafu {
            msg: "Could not find a display".to_string(),
        })?;
        let mut i = 0;
        while let Some(monitor) = display.monitors().item(i) {
            let monitor = monitor
                .to_value()
                .get::<Monitor>()
                .expect("Value needs to be monitor");

            let label = format! {"{}cmx{}cm", monitor.width_mm()/10, monitor.height_mm()/10};
            let button = Button::builder().label(label).build();
            let index = i as i32;
            object
                .imp()
                .monitor_grid
                .attach(&button, index % 4, index / 4, 1, 1);

            button.connect_clicked(clone!(@weak object, @weak monitor => move |_| {
                object.emit_by_name::<()>("monitor", &[&monitor]);
            }));

            i += 1;
        }
        Ok(object)
    }

    /// Signal emitted when monitor button is pressed
    pub fn connect_monitor<F: Fn(&Self, Monitor) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "monitor",
            true,
            glib::closure_local!(|window, monitor| {
                f(window, monitor);
            }),
        )
    }
}
