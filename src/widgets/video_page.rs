use async_channel::Sender;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::ObjectSubclassIsExt;

use crate::config::Campaign;
use crate::program_manager::ControlWindowMessage;
use crate::ui::control_window::Page;

use super::thumbnail_grid::DdThumbnailGrid;

mod imp {
    use glib::subclass::InitializingObject;
    use gtk::subclass::prelude::*;
    use gtk::{glib, Box, Button, CompositeTemplate};
    use gtk::{prelude::*, template_callbacks};

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dragon/display/video_page.ui")]
    pub struct DdVideoPage {
        #[template_child]
        pub content: TemplateChild<Box>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for DdVideoPage {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "DdVideoPage";
        type ParentType = gtk::Widget;
        type Type = super::DdVideoPage;

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
    impl DdVideoPage {
        // In here go the functional button functions
    }

    // Trait shared by all GObjects
    impl ObjectImpl for DdVideoPage {
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
    impl WidgetImpl for DdVideoPage {}
}

glib::wrapper! {
    pub struct DdVideoPage(ObjectSubclass<imp::DdVideoPage>)
        @extends gtk::Widget,
            @implements gtk::Actionable, gtk::Accessible, gtk::Buildable,
                        gtk::ConstraintTarget;
}

impl DdVideoPage {
    pub fn new(campaign: Campaign, sender: Sender<ControlWindowMessage>) -> Self {
        // set all properties
        let object = glib::Object::new::<Self>();
        let imp = object.imp();
        let thumbnail_widget = DdThumbnailGrid::new(campaign, sender, Page::VIDEO);
        thumbnail_widget.set_halign(gtk::Align::Fill);
        thumbnail_widget.set_valign(gtk::Align::Fill);
        thumbnail_widget.set_hexpand(true);
        thumbnail_widget.set_vexpand(true);
        imp.content.append(&thumbnail_widget);

        object
    }
}
