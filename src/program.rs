use gdk4::Monitor;
use glib::subclass::*;
use gtk::gio::prelude::SettingsExt;
use gtk::glib::prelude::*;
use gtk::glib::{self, clone};
use gtk::prelude::{ApplicationExt, GtkWindowExt};
use gtk::subclass::prelude::*;
use snafu::Report;

use crate::errors::DragonDisplayError;
use crate::ui::control_window::DdControlWindow;
use crate::ui::display_window::DdDisplayWindow;
use crate::ui::options::{ColorPreset, DdOptionsWindow};
use crate::{try_emit, APP_ID};
mod imp {

    use std::{cell::OnceCell, sync::OnceLock};

    use super::*;
    #[derive(Default)]
    pub struct DragonDisplayProgram {
        pub control_window: OnceCell<DdControlWindow>,
        pub display_window: OnceCell<DdDisplayWindow>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DragonDisplayProgram {
        const NAME: &'static str = "DdProgram";
        type Type = super::DragonDisplayProgram;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for DragonDisplayProgram {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder("error")
                        .param_types([String::static_type(), bool::static_type()])
                        .build(),
                    Signal::builder("refresh").build(),
                ]
            })
        }
    }
}

glib::wrapper! {
    pub struct DragonDisplayProgram(ObjectSubclass<imp::DragonDisplayProgram>);
}

impl DragonDisplayProgram {
    pub fn new() -> Self {
        let obj = glib::Object::new::<Self>();
        obj
    }

    pub fn run(&self, app: &adw::Application, monitor: &Monitor, campaign: String) {
        let control_window = try_emit!(self, DdControlWindow::new(app, campaign), true);
        let display_window = DdDisplayWindow::new(monitor);
        control_window.present();
        display_window.present();
        display_window.fullscreen_on_monitor(monitor);
        control_window.set_maximized(true);

        control_window.connect_refresh(clone!(@weak self as obj => move |_| {
            obj.emit_by_name::<()>("refresh", &[]);
        }));

        control_window.connect_options(clone!(@weak self as obj, @weak app => move |window| {
            window.set_options_sensitive(false);
            obj.present_options(&app);
        }));

        control_window.connect_fit(clone!(@weak display_window => move |_| {
            display_window.toggle_fit();
        }));

        control_window.connect_reset_display(clone!(@weak display_window => move |_| {
            display_window.reset();
        }));

        control_window.connect_grid(clone!(@weak display_window => move |_| {
            display_window.toggle_grid();
        }));

        control_window.connect_rotate90(clone!(@weak display_window => move |_| {
            display_window.rotate_90();
        }));

        control_window.connect_rotate180(clone!(@weak display_window => move |_| {
            display_window.rotate_180();
        }));

        control_window.connect_rotate270(clone!(@weak display_window => move |_| {
            display_window.rotate_270();
        }));

        control_window.connect_update(clone!(@weak display_window => move |_, path, fow| {
            display_window.set_vtt(path, fow.fow());
        }));

        control_window.connect_image(clone!(@weak display_window => move |_, path| {
            display_window.set_image(path);
        }));

        control_window.connect_video(clone!(@weak display_window => move |_, path| {
            display_window.set_video(path);
        }));

        control_window.connect_error(clone!(@weak self as obj => move |_, msg, fatal| {
            obj.emit_by_name::<()>("error", &[&msg, &fatal]);
        }));

        display_window.connect_error(clone!(@weak self as obj => move |_, msg, fatal| {
            obj.emit_by_name::<()>("error", &[&msg, &fatal]);
        }));

        control_window.connect_close_request(
            clone!(@weak app, @weak display_window, @strong control_window => @default-return glib::Propagation::Proceed, move |_| {
                display_window.destroy();
                control_window.destroy();
                app.quit();
                glib::Propagation::Proceed
            }),
        );

        self.imp()
            .control_window
            .set(control_window)
            .expect("Expected control window to not be set");
        self.imp()
            .display_window
            .set(display_window)
            .expect("Expected control window to not be set");
    }

    /// Update the grid of thumbnails for the pages in the control window of the program
    pub fn update_thumbnail_grid(&self) {
        try_emit!(
            self,
            self.imp()
                .control_window
                .get()
                .expect("Expected control window to be set")
                .update(),
            false
        );
    }

    fn present_options(&self, app: &adw::Application) {
        let options_window = DdOptionsWindow::new(app);

        options_window.connect_confirm(clone!(@weak self as obj => move |window| {
            obj.imp().control_window.get().expect("Expected a control window").set_options_sensitive(true);
            obj.update_thumbnail_grid();
            window.destroy();
        }));

        options_window.connect_color(clone!(@weak self as obj => move |_, color| {
            let color = ColorPreset::from_index(color).to_rgba();
            obj.imp().display_window.get().expect("Expected a display window").update_grid_color(color); 
        }));

        options_window.connect_grid_line_width(clone!(@weak self as obj => move |_, width| {
            obj.imp().display_window.get().expect("Expected a display window").set_gridline_width(width);
        }));

        options_window.connect_close_request(
            clone!(@weak self as obj => @default-return glib::Propagation::Proceed, move |_| {
                let settings = gtk::gio::Settings::new(APP_ID);

                let index = settings.int("grid-color-preset") as u32;
                let color = ColorPreset::from_index(index).to_rgba();
                obj.imp().display_window.get().expect("Expected a display window").update_grid_color(color); 

                let width = settings.double("grid-line-width") as f32;
                obj.imp().display_window.get().expect("Expected a display window").set_gridline_width(width);

                obj.imp().control_window.get().expect("Expected a control window").set_options_sensitive(true);
                glib::Propagation::Proceed
            }),
        );

        options_window.present();
    }

    /**
     * ----------------------------------
     *
     * Signal connect functions
     *
     * --------------------------------
     **/

    /// Signal emitted when an refresh button is pressed
    pub fn connect_refresh<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "refresh",
            true,
            glib::closure_local!(|window| {
                f(window);
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
