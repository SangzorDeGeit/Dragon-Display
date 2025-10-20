use glib::clone;
use gtk::{
    gio, glib, subclass::prelude::ObjectSubclassIsExt, Box, Label, ListItem, SignalListItemFactory,
    SingleSelection, TreeExpander, TreeListModel,
};
use gtk::{prelude::*, ListView};

use super::google_folder_object::GoogleFolderObject;

mod imp {
    use std::sync::OnceLock;

    use glib::subclass::{InitializingObject, Signal};
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use gtk::{glib, CompositeTemplate, ScrolledWindow};

    // Object holding the state
    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dragon/display/folder_tree.ui")]
    pub struct DdGoogleFolderTree {
        #[template_child]
        pub window: TemplateChild<ScrolledWindow>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for DdGoogleFolderTree {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "DdGoogleFolderTree";
        type ParentType = gtk::Widget;
        type Type = super::DdGoogleFolderTree;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_layout_manager_type::<gtk::BoxLayout>();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for DdGoogleFolderTree {
        fn constructed(&self) {
            // Call "constructed" on parent
            self.parent_constructed();
        }

        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }

        fn signals() -> &'static [Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![Signal::builder("folder-selection-changed")
                    .param_types([String::static_type(), String::static_type()])
                    .build()]
            })
        }
    }

    // Trait shared by all widgets
    impl WidgetImpl for DdGoogleFolderTree {}
}

glib::wrapper! {
    pub struct DdGoogleFolderTree(ObjectSubclass<imp::DdGoogleFolderTree>)
        @extends gtk::Widget,
            @implements gtk::Actionable, gtk::Accessible, gtk::Buildable,
                        gtk::ConstraintTarget;
}

impl DdGoogleFolderTree {
    pub fn new(folders: Vec<GoogleFolderObject>) -> Self {
        // set all properties
        let object = glib::Object::new::<Self>();

        // we create a liststore for our root model, this contains one element labelled My drive
        let root_folder = GoogleFolderObject::new("My Drive".to_string(), "root".to_string());

        // any folder that is not a child of any other folder is a child of root
        let mut registered_children: Vec<String> = Vec::new();
        for folder in &folders {
            registered_children.append(&mut folder.children());
        }
        let root_children: Vec<String> = folders
            .iter()
            .filter(|f| !registered_children.contains(&f.id()))
            .map(|f| f.id())
            .collect();
        root_folder.set_children(root_children);
        let root_vec: Vec<GoogleFolderObject> = vec![root_folder];
        let root_store = gio::ListStore::new::<GoogleFolderObject>();

        // add the root folder (as vector) to the root_model
        root_store.extend_from_slice(&root_vec);

        // We create a TreeListModel with as root the root_store variable. Whenever an item gets
        // clicked we want present a new store based on the item that was clicked
        // This model is just to instantiate the data, it does not create any widgets
        let tree_model = TreeListModel::new(root_store, true, false, move |item| {
            let folder_item = item
                .downcast_ref::<GoogleFolderObject>()
                .expect("Found a non folder object when creating the google drive tree");
            let store = gio::ListStore::new::<GoogleFolderObject>();
            // Get all the children from the item that was clicked
            let children_ids = folder_item.children();
            let child_folders: Vec<GoogleFolderObject> = folders
                .iter()
                .filter(|f| children_ids.contains(&f.id()))
                .map(|f| f.clone())
                .collect();
            store.extend_from_slice(&child_folders);
            Some(store.upcast::<gio::ListModel>())
        });

        // To create the widgets, we need a SignalListItemFactory
        let factory = SignalListItemFactory::new();

        // The first step in the factory is to create a new label for every widget that is requested by
        // the model.
        factory.connect_setup(move |_, list_item| {
            let hbox = Box::new(gtk::Orientation::Horizontal, 5);
            let expander = TreeExpander::new();
            let label = Label::new(None);
            hbox.append(&expander);
            hbox.append(&label);
            list_item
                .downcast_ref::<ListItem>()
                .expect("item needs to be a list_item")
                .set_child(Some(&hbox));
        });

        // We want to set the Label of the widget and we want to connect the TreeExpander to the
        // TreeListRow
        factory.connect_bind(clone!(@weak tree_model => move |_, list_item| {
                let listitem = list_item.downcast_ref::<ListItem>().expect("connect_bind in ListitemFactory: item needs to be a ListItem");
                let folder_object = listitem
                    .item()
                    .and_downcast::<GoogleFolderObject>()
                    .expect("Connect_bind in ListItemFactory: item was not a TreeListRow");
                let position = listitem
                    .position();
                let hbox = listitem
                    .child()
                    .and_downcast::<Box>()
                    .expect("Connect_bind in ListItemFactory: child was not a Label");
                let first_child = hbox
                    .first_child()
                    .expect("Connect_bind in ListItemFactory: box did not contain first child");
                let expander = first_child
                    .downcast_ref::<TreeExpander>()
                    .expect("Connect_bind in ListItemFactory: first child was not a tree expander");
                let last_child = hbox
                    .last_child()
                    .expect("Connect_bind in ListItemFactory: box did not contain last child");
                let label = last_child
                    .downcast_ref::<Label>()
                    .expect("Connect_bind in ListItemFactory: first child was not a label");
                let tree_object = tree_model.row(position);
                expander.set_list_row(tree_object.as_ref());
                label.set_label(&folder_object.name());
            }));

        // Only allow one item to be selected
        let selection_model = SingleSelection::new(Some(tree_model));

        selection_model.connect_selection_changed(clone!(@weak object => move |model, _, _| {
            let position = model.selected();
            let binding = model.item(position);
            let folder_object = binding.and_downcast_ref::<GoogleFolderObject>().expect(
                "connect_selection in selection model: item needs to be GoogleFolderObject",
            );
            let folder_name = folder_object.name();
            let folder_id = folder_object.id();
            object.emit_by_name::<()>("folder-selection-changed", &[&folder_id, &folder_name]);
        }));

        let list_view = ListView::new(Some(selection_model), Some(factory));
        list_view.set_hexpand(true);
        list_view.set_vexpand(true);
        list_view.set_valign(gtk::Align::Fill);
        list_view.set_halign(gtk::Align::Fill);
        list_view.set_single_click_activate(false);

        object.imp().window.set_child(Some(&list_view));
        object
    }

    /// Signal emitted when the refresh button is pressed
    pub fn connect_folder_selection_changed<F: Fn(&Self, String, String) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            "folder-selection-changed",
            true,
            glib::closure_local!(|window, id, name| {
                f(window, id, name);
            }),
        )
    }
}
