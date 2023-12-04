// packages for gui
use gtk::{DialogFlags, ResponseType,ApplicationWindow};
use gtk::{Button, Label, Box, glib, Grid, Entry, DropDown, FileChooserNative, Dialog};
use adw::prelude::*;

use std::env;
use std::collections::HashMap;
use std::io::{Error, ErrorKind};



use display_info::DisplayInfo;





use crate::manage_campaign_logic::{write_campaign_to_config, read_campaign_from_config, CampaignData, remove_campaign_from_config};
use crate::run_program;

const CAMPAIGN_MAX_CHAR_LENGTH : u16 = 25;

const SYNCHRONIZATION_OPTIONS : [&str; 2] = ["None", "Google Drive"];

// The "main"/"select campaign" window
pub fn select_campaign_window(app: &adw::Application){

    // use the settings var to store information about the gui
    // let settings = Settings::new(APP_ID);

    let campaign_list = read_campaign_from_config();

    let label = Label::builder()
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();
    let button_add = Button::builder()
        .label("add campaign")
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();
    let button_remove = Button::builder()
        .label("remove campaign")
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();

    let container = Grid::new();
    match campaign_list {
        Some(list) => {
            label.set_text("Select a campaign");
            let mut i = 0;
            container.attach(&button_remove, i, 2, 1, 1);
            for campaign in list {
                i += 1;
                let campaign_button = Button::builder()
                    .label(&campaign.0)
                    .margin_top(6)
                    .margin_bottom(6)
                    .margin_start(6)
                    .margin_end(6)
                    .build();
                campaign_button.connect_clicked(move |_| run_program(&campaign));
                container.attach(&campaign_button, i, 1, 1, 1)
            }
            if i%2 == 0 {
                container.attach(&label, i/2, 0, 2, 1);
            } else {
                container.attach(&label, (i/2)+1, 0, 1, 1);
            }
            container.attach(&button_add, i+1, 2, 1, 1);
        }
        None => {
            label.set_text("You do not have any campaigns yet");
            container.attach(&label, 0, 0, 1, 1);
            container.attach(&button_add, 0, 1, 1, 1);
        }
    }

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Dragon-Display")
        .child(&container)
        .resizable(false)
        .build();
    

    button_add.connect_clicked(glib::clone!(@strong app, @strong window => move |_| {
        window.destroy();
        add_campaign_page_1(&app);
    }));

    button_remove.connect_clicked(glib::clone!(@strong app, @strong window => move |_| {
        window.destroy();
        remove_campaign_window(&app);
    }));

    window.present();

}






// The 'add campaign naming' window
fn add_campaign_page_1(app: &adw::Application) {
    let label = Label::new(Some("Name the campaign"));
    let button_next = Button::builder()
        .label("Next")
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();
    let button_cancel = Button::builder()
        .label("Cancel")
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();
    let entry = Entry::new();
    match env::var("CAMPAIGN_NAME"){
        Ok(name) => entry.set_text(&name),
        Err(_) => entry.set_text(""),
    }
    entry.buffer().set_max_length(Some(CAMPAIGN_MAX_CHAR_LENGTH));

    let page = Grid::new();
    page.attach(&label, 2, 1, 1, 1);
    page.attach(&entry, 2, 2, 1, 1);
    page.attach(&button_next, 3, 3, 1, 1);
    page.attach(&button_cancel, 1, 3, 1, 1);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Add Campaign")
        .child(&page)
        .resizable(false)
        .build();

    button_next.connect_clicked(glib::clone!(@strong window, @strong app => move |_| {
        let input = String::from(entry.text().as_str());
        match valid_name(&input) {
            Ok(_) => {
                env::set_var("CAMPAIGN_NAME", input.as_str());
                add_campaign_page_2(&app);
                window.destroy();
            }
            Err(error) => {
                let msg = format!("Could not add campaign: {}", error.to_string());
                create_error_dialog(&app, &msg)
            }
        }
    }));

    button_cancel.connect_clicked(glib::clone!(@strong app, @strong window => move |_| {
        window.destroy();
        select_campaign_window(&app)
    }));

    window.present();
}



// Checks for valid name
fn valid_name(name: &str) -> Result<(), Error> {
    if name.chars().all(char::is_whitespace) {
        return Err(Error::from(ErrorKind::InvalidInput))
    }

    let campaign_list = match read_campaign_from_config() {
        Some(c) => c,
        None => return Ok(())
    };

    for campaign in campaign_list {
        if campaign.0 == name {
            return Err(Error::from(ErrorKind::AlreadyExists))
        }
    }

    Ok(())
}



// The 'add campaign sync option' window
fn add_campaign_page_2(app: &adw::Application) {
    let label = Label::builder()
       .label("Choose synchronization service")
       .margin_top(6)
       .margin_bottom(6)
       .build();
    let button_previous = Button::builder()
       .label("Back")
       .margin_top(6)
       .margin_bottom(6)
       .margin_start(6)
       .margin_end(6)
       .build();
    let button_next = Button::builder()
       .label("Next")
       .margin_top(6)
       .margin_bottom(6)
       .margin_start(6)
       .margin_end(6)
       .build();
    let button_cancel = Button::builder()
       .label("Cancel")
       .margin_top(6)
       .margin_bottom(6)
       .margin_start(6)
       .margin_end(6)
       .build();
    let dropdown_2 = DropDown::from_strings(&SYNCHRONIZATION_OPTIONS);

    let page_2 = Grid::new();
    page_2.attach(&label, 2, 1, 1, 1);
    page_2.attach(&button_previous, 2, 3, 1, 1);
    page_2.attach(&button_next, 3, 3, 1, 1);
    page_2.attach(&button_cancel, 1, 3, 1, 1);
    page_2.attach(&dropdown_2, 2, 2, 1, 1);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Add Campaign")
        .child(&page_2)
        .resizable(false)
        .build();

    button_previous.connect_clicked(glib::clone!(@strong app, @strong window => move |_| {
        add_campaign_page_1(&app);
        window.destroy();
    }));
    button_cancel.connect_clicked(glib::clone!(@strong app, @strong window => move |_| {
        window.destroy();
        select_campaign_window(&app)
    }));
    button_next.connect_clicked(glib::clone!(@strong window, @strong app => move |_| {
        match dropdown_2.selected(){
            0 => {
                window.destroy();
                add_campaign_page_none(&app);
            },
            _ => {
                window.close();
                add_campaign_page_gd(&app);
            },
        }
    }));

    window.present();
}






// The 'add campaign' window for sync option none
fn add_campaign_page_none(app: &adw::Application) {
    match env::current_dir().unwrap().to_str() {
        Some(path) => env::set_var("CAMPAIGN_PATH", path),
        None => env::set_var("CAMPAIGN_PATH", ""),
    }

    let label = Label::builder()
       .margin_top(6)
       .margin_bottom(6)
       .margin_start(6)
       .margin_end(6)
       .build();
    match env::var("CAMPAIGN_PATH") {
        Ok(path) => label.set_text(format!("Choose location of the image folder.\nCurrent location: {}", &path).as_str()),
        Err(_) => label.set_text("Choose location of image folder"),
    }
    let button_default = Button::builder()
       .label("Use Default")
       .margin_top(6)
       .margin_bottom(6)
       .margin_start(6)
       .margin_end(6)
       .build();    
    let button_choose = Button::builder()
       .label("Choose Location")
       .margin_top(6)
       .margin_bottom(6)
       .margin_start(6)
       .margin_end(6)
       .build();
    let button_previous = Button::builder()
       .label("Back")
       .margin_top(6)
       .margin_bottom(6)
       .margin_start(6)
       .margin_end(6)
       .build();
    let button_finish = Button::builder()
       .label("Finish")
       .margin_top(6)
       .margin_bottom(6)
       .margin_start(6)
       .margin_end(6)
       .build();
    let button_cancel = Button::builder()
       .label("Cancel")
       .margin_top(6)
       .margin_bottom(6)
       .margin_start(6)
       .margin_end(6)
       .build();
    let page_none = Grid::new();
    page_none.attach(&label, 1, 1, 3, 1);
    page_none.attach(&button_previous, 2, 3, 1, 1);
    page_none.attach(&button_cancel, 1, 3, 1, 1);
    page_none.attach(&button_default, 3, 2, 1, 1);
    page_none.attach(&button_choose, 2, 2, 1, 1);
    page_none.attach(&button_finish, 3, 3, 1, 1);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Add Campaign")
        .child(&page_none)
        .resizable(false)
        .build();


    let file_chooser = FileChooserNative::new(
        Some("Choose location of image folder"),
        Some(&window),
        gtk::FileChooserAction::SelectFolder,
        Some("Select"),
        Some("Cancel")
    );

    file_chooser.connect_response(glib::clone!(@strong label => move |file_chooser, response| {
        match response {
            gtk::ResponseType::Accept => {
                match file_chooser.file() {
                    Some(f) => {
                        label.set_text(format!("Choose location of the image folder.\nCurrent location: {}", f.path().unwrap().to_str().unwrap()).as_str());
                        env::set_var("CAMPAIGN_PATH", f.path().unwrap().to_str().unwrap())
                    },
                    None => {}
                }
            },
            _ => {}
        }
    }));

    button_choose.connect_clicked(glib::clone!(@strong file_chooser => move |_| file_chooser.set_visible(true)));
    button_default.connect_clicked(glib::clone!(@strong app, @strong window => move |_| {
        match env::current_dir().unwrap().to_str() {
            Some(path) => add_campaign(&app, path, "None"),
            None => create_error_dialog(&app, "could not find the default directory"),
        }
        window.destroy();
    }));
    button_previous.connect_clicked(glib::clone!(@strong app, @strong window => move |_| {
        add_campaign_page_2(&app);
        window.destroy();
    }));
    button_cancel.connect_clicked(glib::clone!(@strong app, @strong window => move |_| {
        window.destroy();
        select_campaign_window(&app)
    }));
    button_finish.connect_clicked(glib::clone!(@strong app, @strong window => move |_| {
        match env::var("CAMPAIGN_PATH") {
            Ok(path) => add_campaign(&app, &path, "None"),
            Err(_) => create_error_dialog(&app, "Select a location for the image folder")
        };
        window.destroy();
    }));


    window.present();
}







// The 'add campaign' window for sync option google drive
fn add_campaign_page_gd(app: &adw::Application) {
    let page_gd = Grid::new();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Add Campaign")
        .child(&page_gd)
        .resizable(false)
        .build();

    window.present();
}






// The 'remove campaign' window
fn remove_campaign_window(app: &adw::Application){
    let campaign_list = read_campaign_from_config();

    let label = Label::builder()
        .label("Select the campaign you want to remove")
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();
    let button_cancel = Button::builder()
        .label("Cancel")
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();


    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Dragon-Display")
        .resizable(false)
        .build();

    let container = Grid::new();
    match campaign_list {
        Some(list) => {
            let mut i = 0;
            for campaign in list {
                let campaign_button = Button::builder()
                    .label(&campaign.0)
                    .margin_top(6)
                    .margin_bottom(6)
                    .margin_start(6)
                    .margin_end(6)
                    .build();
                let campaign_name = campaign.0.clone();
                campaign_button.connect_clicked(glib::clone!(@strong app, @strong window, @strong campaign_name =>move |_| {
                    let confirm_window = remove_campaign_window_confirm(&window, campaign_name.as_str());
                    confirm_window.present();
                    confirm_window.connect_response(glib::clone!(@strong app, @strong confirm_window, @strong campaign_name, @strong window => move |_, response| {
                        match response {
                            ResponseType::Yes => {
                                remove_campaign(&app, campaign_name.as_str());
                                confirm_window.destroy();
                                window.destroy()
                            },
                            _ => confirm_window.destroy()
                        }
                    }));
                }));
                container.attach(&campaign_button, i, 1, 1, 1);
                i += 1;
            }
            i -= 1;
            if i%2 == 0 {
                container.attach(&label, i/2, 0, 1, 1);
                container.attach(&button_cancel, i/2, 2, 1, 1);
            } else {
                container.attach(&label, i/2, 0, 2, 1);
                container.attach(&button_cancel, i/2, 2, 2, 1);
            }
        }
        None => {
            create_error_dialog(app, "There are no campaigns to remove!");
        }
    }

    window.set_content(Some(&container));

    button_cancel.connect_clicked(glib::clone!(@strong app, @strong window => move |_| {
        window.destroy();
        select_campaign_window(&app)
    }));

    window.present();
}





// A modal window to confirm the removal of a campaign
fn remove_campaign_window_confirm(window: &adw::ApplicationWindow, campaign_name: &str) -> Dialog {
    let msg = format!("delete {}?", campaign_name);
    let buttons = [("On second though, No", ResponseType::No), ("Do it!", ResponseType::Yes)];
    let window = Dialog::with_buttons(Some(msg), Some(window), DialogFlags::MODAL, &buttons);
    return window;
}




// function to create an error dialog
pub fn create_error_dialog(app: &adw::Application, msg: &str) {
    let label = Label::builder()
        .label(msg)
        .margin_bottom(6)
        .margin_top(6)
        .margin_start(6)
        .margin_end(6)
        .build();
    let button = Button::builder()
        .label("Ok")
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();

    let container = Box::new(gtk::Orientation::Vertical, 10);
    container.append(&label);
    container.append(&button);

    let window = Dialog::builder()
        .application(app)
        .child(&container)
        .resizable(false)
        .build();
    window.set_modal(true);

    button.connect_clicked(glib::clone!(@strong window => move |_| {
        window.destroy()
    }));

    window.present()
}







// This function is called by the gui modules to create the campaign
fn add_campaign(app: &adw::Application, path: &str, sync_option: &str){
    let name = match env::var("CAMPAIGN_NAME") {
        Ok(n) => n,
        Err(_) => {
            create_error_dialog(app, "Could not find a campaign name");
            select_campaign_window(app);
            return;
        }    
    };

    let campaign_values = CampaignData {
        path : path.to_string(),
        sync_option: sync_option.to_string()
    };

    let mut campaign = HashMap::new();
    campaign.insert(name, campaign_values);

    match write_campaign_to_config(campaign) {
        Ok(_) => select_campaign_window(app),
        Err(error) => {
            let msg = format!("Could not add campaign: {}", error.to_string());
            create_error_dialog(app, &msg.as_str());
            select_campaign_window(app)
        }
    }   
}




// This function is called by the gui modules to remove given campaign
// TODO: any envirnoment variables for sync services should be removed
fn remove_campaign(app: &adw::Application, campaign_name: &str) {
    match remove_campaign_from_config(campaign_name) {
        Ok(_) => select_campaign_window(app),
        Err(error) => {
            let msg = format!("Could not remove campaign: {}", error.to_string());
            create_error_dialog(app, &msg.as_str());
            select_campaign_window(app)
        }
    }
}
    