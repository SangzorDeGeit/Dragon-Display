pub mod add_campaign;
pub mod error;
pub mod google_drive;
pub mod remove_campaign;
pub mod select_campaign;
pub mod select_monitor;

use gtk::prelude::*;
use gtk::{Button, Label};

trait CustomMargin {
    fn set_margin_all(&self, margin: i32);
}

impl CustomMargin for Button {
    fn set_margin_all(&self, margin: i32) {
        self.set_margin_end(margin);
        self.set_margin_start(margin);
        self.set_margin_top(margin);
        self.set_margin_bottom(margin);
    }
}

impl CustomMargin for Label {
    fn set_margin_all(&self, margin: i32) {
        self.set_margin_end(margin);
        self.set_margin_start(margin);
        self.set_margin_top(margin);
        self.set_margin_bottom(margin);
    }
}
