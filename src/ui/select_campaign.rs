use adw::Application;
use gtk::glib::clone;
use gtk::prelude::ObjectExt;
use gtk::subclass::prelude::ObjectSubclassIsExt;
use gtk::{gio, glib};
use gtk::{prelude::*, Button};

use crate::campaign::DdCampaign;
use crate::config::Campaign;

mod imp {
    use std::cell::RefCell;
    use std::sync::OnceLock;

    use glib::subclass::{InitializingObject, Signal};
    use gtk::glib::object::ObjectExt;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::{glib, template_callbacks, Box, Button, CompositeTemplate, Grid, Label};

    use crate::campaign::DdCampaign;

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dragon/display/select_campaign.ui")]
    pub struct SelectCampaignWindow {
        #[template_child]
        pub select_message: TemplateChild<Label>,
        #[template_child]
        pub campaign_grid: TemplateChild<Grid>,
        #[template_child]
        pub remove_add_box: TemplateChild<Box>,
        pub message: RefCell<String>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for SelectCampaignWindow {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "DdSelectCampaignWindow";
        type Type = super::SelectCampaignWindow;
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
    impl SelectCampaignWindow {
        #[template_callback]
        fn handle_remove(&self, _: Button) {
            let obj = self.obj();
            obj.emit_by_name::<()>("remove-campaign", &[]);
        }

        #[template_callback]
        fn handle_add(&self, _: Button) {
            let obj = self.obj();
            obj.emit_by_name::<()>("add-campaign", &[]);
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for SelectCampaignWindow {
        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder("remove-campaign").build(),
                    Signal::builder("add-campaign").build(),
                    Signal::builder("campaign")
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
    impl WidgetImpl for SelectCampaignWindow {}

    // Trait shared by all windows
    impl WindowImpl for SelectCampaignWindow {}

    // Trait shared by all application windows
    impl ApplicationWindowImpl for SelectCampaignWindow {}
}

glib::wrapper! {
    pub struct SelectCampaignWindow(ObjectSubclass<imp::SelectCampaignWindow>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
            @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                        gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl SelectCampaignWindow {
    pub fn new(app: &Application, campaign_list: Vec<Campaign>) -> Self {
        // set all properties
        let object = glib::Object::new::<Self>();
        let imp = object.imp();
        object.set_property("application", app);

        if campaign_list.is_empty() {
            imp.select_message.set_text("You have no campaigns yet");
            imp.remove_add_box.remove(
                &imp.remove_add_box
                    .first_child()
                    .expect("Could not find the remove button"),
            );
        } else {
            imp.select_message.set_text("Select a campaign");
        }

        let mut index = 0;
        for campaign in campaign_list {
            let button = Button::builder().label(&campaign.name).build();
            imp.campaign_grid
                .attach(&button, index % 4, index / 4, 1, 1);
            let campaign = DdCampaign::from(campaign);
            button.connect_clicked(
                clone!(@weak object => move |_| object.emit_by_name::<()>("campaign", &[&campaign])),
            );
            index += 1;
        }
        object
    }

    /// The signal emitted when the remove button is clicked
    pub fn connect_remove_campaign<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "remove-campaign",
            true,
            glib::closure_local!(|window| {
                f(window);
            }),
        )
    }

    /// The signal emitted when the add button is clicked
    pub fn connect_add_campaign<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "add-campaign",
            true,
            glib::closure_local!(|window| {
                f(window);
            }),
        )
    }

    /// Signal emitted when a campaign button is clicked
    pub fn connect_campaign<F: Fn(&Self, DdCampaign) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            "campaign",
            true,
            glib::closure_local!(|window, campaign| {
                f(window, campaign);
            }),
        )
    }
}
