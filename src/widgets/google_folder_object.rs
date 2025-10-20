use gtk::glib;
use gtk::subclass::prelude::*;

mod imp {
    use super::*;
    use gtk::glib;
    use std::sync::OnceLock;

    #[derive(Default)]
    pub struct GoogleFolderObject {
        pub name: OnceLock<String>,
        pub id: OnceLock<String>,
        pub children: OnceLock<Vec<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for GoogleFolderObject {
        const NAME: &'static str = "DdGoogleFolderObject";
        type Type = super::GoogleFolderObject;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for GoogleFolderObject {}
}

glib::wrapper! {
    pub struct GoogleFolderObject(ObjectSubclass<imp::GoogleFolderObject>);
}

impl GoogleFolderObject {
    pub fn new(id: String, name: String) -> Self {
        let object = glib::Object::new::<Self>();
        object.imp().id.set(id).expect("Expected no id");
        object.imp().name.set(name).expect("Exptected no name");
        object
    }

    /// Get the id of the folder, this function panics if id is not initialized
    pub fn id(&self) -> String {
        self.imp().id.get().expect("Expected id to be set").clone()
    }

    /// Get the name of the folder, this function panics if name is not initialized
    pub fn name(&self) -> String {
        self.imp()
            .name
            .get()
            .expect("Expected name to be set")
            .clone()
    }

    /// Get the children of the folder, this function panics if children is not initialized
    pub fn children(&self) -> Vec<String> {
        self.imp()
            .children
            .get()
            .expect("Expected children to be set")
            .clone()
    }

    /// Set the children variable of the folder object. This function panics if the children
    /// was already set
    pub fn set_children(&self, children: Vec<String>) {
        self.imp()
            .children
            .set(children)
            .expect("Expected no children");
    }
}
