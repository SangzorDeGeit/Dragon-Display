use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use gdk4::builders::RGBABuilder;
use gdk4::Texture;
use gtk::glib::object::ObjectExt;
use gtk::glib::{clone, Bytes};
use gtk::graphene::{Rect, Size};
use gtk::prelude::{SnapshotExt, TextureExt, WidgetExt};
use gtk::{glib, GestureClick};
use snafu::{Report, ResultExt};
use std::cell::{Cell, OnceCell};
use std::rc::Rc;
use std::sync::OnceLock;
use vtt_rust::fog_of_war::Operation;
use vtt_rust::Coordinate;
use vtt_rust::{FogOfWar, VTT};

use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;

use crate::errors::{DragonDisplayError, GlibSnafu};
use crate::fogofwar::DdFogOfWar;

mod imp {
    use std::cell::RefCell;

    use gdk4::Texture;
    use gtk::{
        glib::{
            subclass::{InitializingObject, Signal},
            types::StaticType,
        },
        template_callbacks, Button, Picture,
    };

    use crate::fogofwar::DdFogOfWar;

    use super::*;

    #[derive(CompositeTemplate, Default)]
    #[template(resource = "/dragon/display/vtt_area.ui")]
    pub struct DdVttArea {
        #[template_child]
        pub image: TemplateChild<Picture>,
        pub vtt: RefCell<Option<VTT>>,
        pub texture: RefCell<Option<Texture>>,
    }

    // The central trait for subclassing a GObject
    #[glib::object_subclass]
    impl ObjectSubclass for DdVttArea {
        // `NAME` needs to match `class` attribute of template
        const NAME: &'static str = "DdVttArea";
        type Type = super::DdVttArea;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.set_layout_manager_type::<gtk::BoxLayout>();
            klass.bind_template_callbacks();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    #[template_callbacks]
    impl DdVttArea {
        #[template_callback]
        fn handle_show_all(&self, _: Button) {
            self.obj().fow_show_all();
        }

        #[template_callback]
        fn handle_hide_all(&self, _: Button) {
            self.obj().fow_hide_all();
        }
    }

    // Trait shared by all GObjects
    impl ObjectImpl for DdVttArea {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder("update")
                        .param_types([DdFogOfWar::static_type()])
                        .build(),
                    Signal::builder("error")
                        .param_types([String::static_type(), bool::static_type()])
                        .build(),
                ]
            })
        }

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
    impl WidgetImpl for DdVttArea {}
}

glib::wrapper! {
    pub struct DdVttArea(ObjectSubclass<imp::DdVttArea>)
        @extends gtk::Widget,
            @implements gtk::Actionable, gtk::Accessible, gtk::Buildable,
                        gtk::ConstraintTarget;
}

impl DdVttArea {
    pub fn new(path: &str) -> Result<Self, DragonDisplayError> {
        let object = glib::Object::new::<Self>();
        // say image width = 2000 pix
        // real image = 10x10 with 100 pix per grid = 1000 pix
        // if we click on (100, 100), in the real image we click on (50, 50)
        //
        // aspect ratio = real_image/image;
        // coordinate*ratio = real image coordinate
        let mut vtt = vtt_rust::open_vtt(path).expect("Could not open vtt");
        let image = match vtt.take_image() {
            Ok(i) => i,
            Err(_) => {
                return Err(DragonDisplayError::Other {
                    msg: "Failed to get image from vtt".to_string(),
                })
            }
        };
        let bytes = Bytes::from(&image);
        let texture = Texture::from_bytes(&bytes).context(GlibSnafu {
            msg: "Failed to load vtt file",
        })?;
        object.imp().image.set_paintable(Some(&texture));
        object.imp().texture.replace(Some(texture));

        let pressed = Rc::new(Cell::new(0));
        let xcoord = Rc::new(Cell::new(0.));
        let ycoord = Rc::new(Cell::new(0.));
        let scale_factor_x: Rc<OnceCell<f64>> = Rc::new(OnceCell::new());
        let scale_factor_y: Rc<OnceCell<f64>> = Rc::new(OnceCell::new());
        let grid_x = vtt.size().x;
        let grid_y = vtt.size().y;

        let clickable_area = GestureClick::builder().button(0).build();
        clickable_area.connect_pressed(clone!(@strong scale_factor_x, @strong scale_factor_y, @strong xcoord, @strong ycoord, @strong pressed, @weak object => move |_, n, x, y| {
            if scale_factor_x.get().is_none() {
                let width = object.imp().image.width();
                let height = object.imp().image.height();
                scale_factor_x.set(width as f64 / grid_x).expect("Expected scale_factor_x to be empty");
                scale_factor_y.set(height as f64 / grid_y).expect("Expected scale_factor_y to be empty");
            }
            pressed.set(n);
            xcoord.set(x/scale_factor_x.get().expect("Expected scale_factor_x to be empty"));
            ycoord.set(y/scale_factor_y.get().expect("Expected scale_factor_y to be empty"));
        }));

        clickable_area.connect_stopped(
            clone!(@strong scale_factor_x, @strong scale_factor_y, @strong xcoord, @strong ycoord, @strong pressed, @weak object => move |_| {
                if pressed.get() > 0 {
                    pressed.set(pressed.get()+1);
                    let coord = Coordinate {
                        x: xcoord.get(),
                        y: ycoord.get(),
                    };
                    object.fow_hide(coord);
                }
            }),
        );

        clickable_area.connect_released(
            clone!(@strong scale_factor_x, @strong scale_factor_y, @strong xcoord, @strong ycoord, @strong pressed, @weak object => move |_,n,_,_| {
                let old_n = pressed.get();
                if n == old_n {
                    let coord = Coordinate {
                        x: xcoord.get(),
                        y: ycoord.get(),
                    };
                    object.fow_show(coord);
                }
                pressed.set(0);
            }),
        );

        object.imp().image.add_controller(clickable_area);
        object.imp().vtt.replace(Some(vtt));
        Ok(object)
    }

    /// Hide the entire vtt image
    pub fn fow_hide_all(&self) {
        {
            let mut borrowed = self.imp().vtt.borrow_mut();
            let vtt = match borrowed.as_mut() {
                Some(v) => v,
                None => return,
            };
            vtt.fow_hide_all();
        }
        self.redraw();
    }

    /// Show the entire vtt image
    pub fn fow_show_all(&self) {
        {
            let mut borrowed = self.imp().vtt.borrow_mut();
            let vtt = match borrowed.as_mut() {
                Some(v) => v,
                None => return,
            };
            vtt.fow_show_all();
        }
        self.redraw();
    }

    pub fn fow_show(&self, point: Coordinate) {
        {
            let mut borrowed = self.imp().vtt.borrow_mut();
            let vtt = match borrowed.as_mut() {
                Some(v) => v,
                None => return,
            };
            let _ = vtt.fow_change(point, Operation::SHOW, true, true);
        }
        self.redraw();
        //redraw
    }

    pub fn fow_hide(&self, point: Coordinate) {
        {
            let mut borrowed = self.imp().vtt.borrow_mut();
            let vtt = match borrowed.as_mut() {
                Some(v) => v,
                None => return,
            };
            let _ = vtt.fow_change(point, Operation::HIDE, true, true);
        }
        self.redraw();
    }

    /// Redraw vtt data to current image
    fn redraw(&self) {
        let binding = &*self.imp().texture.borrow();
        let texture = match binding {
            Some(t) => t,
            None => {
                return;
            }
        };
        let fow = self.fow();
        let snapshot = gtk::Snapshot::new();
        snapshot.save();
        let red = RGBABuilder::new()
            .red(10.)
            .blue(0.)
            .green(0.)
            .alpha(0.05)
            .build();
        let width = texture.width() as f32;
        let height = texture.height() as f32;
        snapshot.append_texture(texture, &Rect::new(0.0, 0.0, width, height));
        for rectangle in fow.get_rectangles() {
            let width = (rectangle.bottomright.x - rectangle.topleft.x) + 1;
            let height = (rectangle.bottomright.y - rectangle.topleft.y) + 1;
            let rect = Rect::new(
                rectangle.topleft.x as f32,
                rectangle.topleft.y as f32,
                width as f32,
                height as f32,
            );
            snapshot.append_color(&red, &rect);
        }
        snapshot.restore();
        let width = texture.width() as f32;
        let height = texture.height() as f32;
        let paintable = snapshot.to_paintable(Some(&Size::new(width, height)));
        self.imp().image.set_paintable(paintable.as_ref());
        let fow = DdFogOfWar::new(fow);
        self.emit_by_name::<()>("update", &[&fow]);
    }

    /// Get the fog of war area, panics if there is no vtt loaded
    pub fn fow(&self) -> FogOfWar {
        let borrowed = self.imp().vtt.borrow();
        let vtt = match borrowed.as_ref() {
            Some(v) => v,
            None => panic!("No vtt found"),
        };
        vtt.get_fow().clone()
    }

    /// Signal emitted when an error occurs
    pub fn connect_pressed<F: Fn(&Self, i32, f64, f64) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            "pressed",
            true,
            glib::closure_local!(|area, n, x, y| {
                f(area, n, x, y);
            }),
        )
    }

    /// Signal emitted when an error occurs
    pub fn connect_stopped<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "stopped",
            true,
            glib::closure_local!(|area| {
                f(area);
            }),
        )
    }

    /// Signal emitted when the vtt has an update. This signal does not mean the vtt needs to be
    /// updated but that something has changed in the vtt file
    pub fn connect_update<F: Fn(&Self, DdFogOfWar) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            "update",
            true,
            glib::closure_local!(|area, fow| {
                f(area, fow);
            }),
        )
    }

    /// Signal emitted when an error occurs
    pub fn connect_released<F: Fn(&Self, i32) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "released",
            true,
            glib::closure_local!(|area, n| {
                f(area, n);
            }),
        )
    }

    /// Emit an error message based on the input error
    pub fn emit_error(&self, err: DragonDisplayError, fatal: bool) {
        let msg = Report::from_error(err).to_string();
        self.emit_by_name::<()>("error", &[&msg, &fatal]);
    }

    /// Signal emitted when an error occurs
    pub fn connect_error<F: Fn(&Self, String, bool) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            "error",
            true,
            glib::closure_local!(|window, msg, fatal| {
                f(window, msg, fatal);
            }),
        )
    }
}
