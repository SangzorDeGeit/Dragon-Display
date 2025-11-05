use gtk::{glib, graphene::Rect, subclass::prelude::*};
use vtt_rust::FogOfWar;

mod imp {

    use std::cell::OnceCell;

    use gtk::graphene::Rect;

    use super::*;

    #[derive(Default)]
    pub struct DdFogOfWar {
        pub fogofwar: OnceCell<Vec<Rect>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DdFogOfWar {
        const NAME: &'static str = "DdFogOfWar";
        type Type = super::DdFogOfWar;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for DdFogOfWar {}
}

glib::wrapper! {
    pub struct DdFogOfWar(ObjectSubclass<imp::DdFogOfWar>);
}

impl DdFogOfWar {
    pub fn new(fogofwar: FogOfWar) -> Self {
        let obj = glib::Object::new::<Self>();
        let rects: Vec<Rect> = fogofwar
            .get_rectangles()
            .iter_mut()
            .map(|f| {
                let width = (f.bottomright.x - f.topleft.x) + 1;
                let height = (f.bottomright.y - f.topleft.y) + 1;
                Rect::new(
                    f.topleft.x as f32,
                    f.topleft.y as f32,
                    width as f32,
                    height as f32,
                )
            })
            .collect();
        obj.imp().fogofwar.set(rects);
        obj
    }

    pub fn fow(&self) -> Vec<Rect> {
        self.imp()
            .fogofwar
            .get()
            .expect("Object must have a fog of war")
            .to_vec()
    }
}

impl Default for DdFogOfWar {
    fn default() -> Self {
        glib::Object::new::<Self>()
    }
}
