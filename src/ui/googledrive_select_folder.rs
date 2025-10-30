use gtk::glib::clone;
use gtk::prelude::ObjectExt;
use gtk::prelude::*;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{gio, glib};

use crate::widgets::google_folder_object::GoogleFolderObject;
use crate::widgets::google_folder_tree::DdGoogleFolderTree;

mod imp {

    use gtk::glib::subclass::Signal;
    use gtk::prelude::*;
    use std::cell::RefCell;
    use std::sync::OnceLock;

    use glib::subclass::InitializingObject;
    use gtk::subclass::prelude::*;
    use gtk::{glib, template_callbacks, Box, Button, CompositeTemplate, Label};

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dragon/display/googledrive_select_folder.ui")]
    pub struct DdGoogleFolderSelectWindow {
        #[template_child]
        pub selection_label: TemplateChild<Label>,
        #[template_child]
        pub select_widget: TemplateChild<Box>,
        #[template_child]
        pub choose_button: TemplateChild<Button>,
        #[template_child]
        pub refresh_button: TemplateChild<Button>,
        pub selection: RefCell<String>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for DdGoogleFolderSelectWindow {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "DdGoogleFolderSelectWindow";
        type Type = super::DdGoogleFolderSelectWindow;
        type ParentType = gtk::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_callbacks()
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[template_callbacks]
    impl DdGoogleFolderSelectWindow {
        #[template_callback]
        fn handle_cancel(&self, _: Button) {
            self.obj().emit_by_name::<()>("cancel", &[]);
        }

        #[template_callback]
        fn handle_refresh(&self, _: Button) {
            self.obj().emit_by_name::<()>("refresh", &[]);
        }

        #[template_callback]
        fn handle_choose(&self, _: Button) {
            let id = self.selection.take();
            self.obj().emit_by_name::<()>("choose", &[&id]);
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for DdGoogleFolderSelectWindow {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder("choose")
                        .param_types([String::static_type()])
                        .build(),
                    Signal::builder("cancel").build(),
                    Signal::builder("refresh").build(),
                ]
            })
        }
        fn constructed(&self) {
            // Call "constructed" on parent
            self.parent_constructed();
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for DdGoogleFolderSelectWindow {}

    // Trait shared by all windows
    impl WindowImpl for DdGoogleFolderSelectWindow {}

    // Trait shared by all application windows
    impl ApplicationWindowImpl for DdGoogleFolderSelectWindow {}
}

glib::wrapper! {
    pub struct DdGoogleFolderSelectWindow(ObjectSubclass<imp::DdGoogleFolderSelectWindow>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
            @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                        gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl DdGoogleFolderSelectWindow {
    pub fn new(app: &gtk::Application, folders: Vec<GoogleFolderObject>) -> Self {
        // set all properties
        let object = glib::Object::new::<Self>();
        object.set_property("application", app);
        let folder_tree = DdGoogleFolderTree::new(folders);
        object.imp().select_widget.append(&folder_tree);

        folder_tree.connect_folder_selection_changed(clone!(@weak object => move |_, id, name| {
            object.imp().selection_label.set_text(&name);
            object.imp().selection.replace(id);
            object.imp().choose_button.set_sensitive(true);
        }));

        object
    }

    /// Signal emitted when the refresh button is pressed
    pub fn connect_refresh<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "refresh",
            true,
            glib::closure_local!(|window| {
                f(window);
            }),
        )
    }

    /// Signal emitted when the cancel button is pressed
    pub fn connect_cancel<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "cancel",
            true,
            glib::closure_local!(|window| {
                f(window);
            }),
        )
    }

    /// Signal emitted when the choose button is pressed containing the id chosen
    pub fn connect_choose<F: Fn(&Self, String) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "choose",
            true,
            glib::closure_local!(|window, id| {
                f(window, id);
            }),
        )
    }
}
