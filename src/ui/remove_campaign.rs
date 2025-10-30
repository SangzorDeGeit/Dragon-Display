use gtk::glib::clone;
use gtk::prelude::ObjectExt;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{gio, glib};
use gtk::{prelude::*, Button};

use crate::campaign::DdCampaign;
use crate::config::Campaign;

mod imp {
    use std::sync::OnceLock;

    use glib::subclass::InitializingObject;
    use gtk::glib::subclass::Signal;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::{glib, template_callbacks, Button, CompositeTemplate, Grid};

    use crate::campaign::DdCampaign;

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dragon/display/remove_campaign.ui")]
    pub struct RemoveCampaignWindow {
        #[template_child]
        pub campaign_grid: TemplateChild<Grid>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for RemoveCampaignWindow {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "DdRemoveCampaignWindow";
        type Type = super::RemoveCampaignWindow;
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
    impl RemoveCampaignWindow {
        #[template_callback]
        fn handle_cancel(&self, _: Button) {
            let obj = self.obj();
            obj.emit_by_name::<()>("cancel", &[]);
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for RemoveCampaignWindow {
        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder("cancel").build(),
                    Signal::builder("remove")
                        .param_types([DdCampaign::static_type()])
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
    impl WidgetImpl for RemoveCampaignWindow {}

    // Trait shared by all windows
    impl WindowImpl for RemoveCampaignWindow {}

    // Trait shared by all application windows
    impl ApplicationWindowImpl for RemoveCampaignWindow {}
}

glib::wrapper! {
    pub struct RemoveCampaignWindow(ObjectSubclass<imp::RemoveCampaignWindow>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
            @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                        gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl RemoveCampaignWindow {
    pub fn new(app: &gtk::Application, campaign_list: Vec<Campaign>) -> Self {
        // set all properties
        let object = glib::Object::new::<Self>();
        let imp = object.imp();
        object.set_property("application", app);

        let mut index = 0;
        for campaign in campaign_list {
            let button = Button::builder().label(&campaign.name).build();
            imp.campaign_grid
                .attach(&button, index % 4, index / 4, 1, 1);

            let campaign = DdCampaign::from(campaign);
            button.connect_clicked(
                clone!(@weak object => move |_| object.emit_by_name::<()>("remove", &[&campaign])),
            );

            index += 1;
        }
        object
    }

    /// Signal emitted when a campaign button is clicked to be removed
    pub fn connect_remove<F: Fn(&Self, DdCampaign) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            "remove",
            true,
            glib::closure_local!(|window, campaign| {
                f(window, campaign);
            }),
        )
    }

    /// The signal emitted when the cancel button is clicked
    pub fn connect_cancel<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "cancel",
            true,
            glib::closure_local!(|window| {
                f(window);
            }),
        )
    }
}
