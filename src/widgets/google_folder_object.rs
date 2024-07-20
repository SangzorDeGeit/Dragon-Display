use glib::Object;
use gtk::glib;

mod imp {
    use gtk::glib;
    use glib::Properties;
    use std::cell::RefCell;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;

    #[derive(Properties, Default)]
    #[properties(wrapper_type = super::GoogleFolderObject)] 
    pub struct GoogleFolderObject {
        #[property(get, set)]
        name: RefCell<String>,
        #[property(get, set)]
        id: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GoogleFolderObject {
        const NAME: &'static str = "DragonDisplayGoogleFolderObject";
        type Type = super::GoogleFolderObject;
    }

    #[glib::derived_properties]
    impl ObjectImpl for GoogleFolderObject {
    }
}

glib::wrapper! {
    pub struct GoogleFolderObject(ObjectSubclass<imp::GoogleFolderObject>);
}

impl GoogleFolderObject {
    pub fn new(name: String, id: String) -> Self {
        Object::builder()
            .property("name", name)
            .property("id", id)
            .build()
    }
}
