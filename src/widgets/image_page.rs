use async_channel::Sender;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::ObjectSubclassIsExt;

use crate::config::Campaign;
use crate::ui::control_window::UpdateDisplayMessage;

use super::thumbnail_grid::DdThumbnailGrid;

mod imp {
    use glib::subclass::InitializingObject;
    use gtk::subclass::prelude::*;
    use gtk::{glib, Box, Button, CompositeTemplate};
    use gtk::{prelude::*, template_callbacks};

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dragon/display/image_page.ui")]
    pub struct DdImagePage {
        #[template_child]
        pub content: TemplateChild<Box>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for DdImagePage {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "DdImagePage";
        type ParentType = gtk::Widget;
        type Type = super::DdImagePage;

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
    impl DdImagePage {
        #[template_callback]
        fn handle_rotate_90(&self, _: Button) {
            todo!("implement this function");
        }

        #[template_callback]
        fn handle_rotate_180(&self, _: Button) {
            todo!("implement this function");
        }

        #[template_callback]
        fn handle_fit(&self, _: Button) {
            todo!("implement this function");
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for DdImagePage {
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
    impl WidgetImpl for DdImagePage {}
}

glib::wrapper! {
    pub struct DdImagePage(ObjectSubclass<imp::DdImagePage>)
        @extends gtk::Widget,
            @implements gtk::Actionable, gtk::Accessible, gtk::Buildable,
                        gtk::ConstraintTarget;
}

impl DdImagePage {
    pub fn new(campaign: Campaign, sender: Sender<UpdateDisplayMessage>) -> Self {
        // set all properties
        let object = glib::Object::new::<Self>();
        let imp = object.imp();
        let thumbnail_widget = DdThumbnailGrid::new(campaign, sender);
        thumbnail_widget.set_halign(gtk::Align::Fill);
        thumbnail_widget.set_valign(gtk::Align::Fill);
        thumbnail_widget.set_hexpand(true);
        thumbnail_widget.set_vexpand(true);
        imp.content.append(&thumbnail_widget);

        object
    }
}
