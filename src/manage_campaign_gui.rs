// packages for gui
use gtk::{prelude::*, ApplicationWindow};
use gtk::{Application, Button, Label, Box, glib, Grid, Entry, DropDown, FileChooserNative, Dialog};

use std::env;

use crate::manage_campaign_logic::{write_campaign_to_config, self};

const CAMPAIGN_MAX_CHAR_LENGTH : u16 = 25;

const SYNCHRONIZATION_OPTIONS : [&str; 2] = ["None", "Google Drive"];

// The "main"/"select campaign" window
pub fn select_campaign_window(app: &Application){
    //read config -> list of campaigns

    // Make the widget elements
    let add_button = Button::builder()
        .label("add campaign")
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();

    let remove_button = Button::builder()
        .label("remove campaign")
        .margin_top(6)
        .margin_bottom(6)
        .margin_start(6)
        .margin_end(6)
        .build();

    //make amount a button for each campaign

    //Make a box to put all the buttons in
    let container = Box::new(gtk::Orientation::Horizontal, 100);
    container.append(&add_button);
    container.append(&remove_button);

    //build the window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Dragon-Display")
        .child(&container)
        .build();


    // Connect the widgets to actions
    add_button.connect_clicked(glib::clone!(@strong app, @strong window => move |_| {
        window.destroy();
        add_campaign_page_1(&app);
    }));

    //connect each button to sync the correct pictures and run the program

    window.present();
}






// The 'add campaign' window
fn add_campaign_page_1(app: &Application) {
    //setup page 1
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
    // Get text from this widget: entry.text().as_str()

    let page = Grid::new();
    page.attach(&label, 2, 1, 1, 1);
    page.attach(&entry, 2, 2, 1, 1);
    page.attach(&button_next, 3, 3, 1, 1);
    page.attach(&button_cancel, 1, 3, 1, 1);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Add Campaign")
        .child(&page)
        .build();

    button_next.connect_clicked(glib::clone!(@strong window, @strong app => move |_| {
        let input = String::from(entry.text().as_str());
        if input.chars().all(char::is_whitespace) {
            create_error_dialog(&app, "name may not be empty");
        } else {
            env::set_var("CAMPAIGN_NAME", input.as_str());
            add_campaign_page_2(&app);
            window.destroy();
        }

    }));
    button_cancel.connect_clicked(glib::clone!(@strong app, @strong window => move |_| {
        window.destroy();
        select_campaign_window(&app)
    }));

    window.present();
}







fn add_campaign_page_2(app: &Application) {
    // setup page 2
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







fn add_campaign_page_none(app: &Application) {
    // setup page (3) none
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







   // setup page (3) google drive
fn add_campaign_page_gd(app: &Application) {
    let page_gd = Grid::new();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Add Campaign")
        .child(&page_gd)
        .build();

    window.present();
}






// The 'remove campaign' window
fn remove_campaign_window(){
    todo!();
}






pub fn create_error_dialog(app: &Application, msg: &str) {
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
        .build();
    window.set_modal(true);

    button.connect_clicked(glib::clone!(@strong window => move |_| {
        window.destroy()
    }));

    window.show()
}







// This function is called by the gui modules to create the campaign
fn add_campaign(app: &Application, path: &str, sync_option: &str){
    match env::var("CAMPAIGN_NAME") {
        Ok(name) => {
            let campaign = manage_campaign_logic::Campaign {
                name: String::from(name),
                path: String::from(path),
                sync_option: String::from(sync_option)
            };
            write_campaign_to_config(campaign);
        },
        Err(_) => create_error_dialog(app, "Could not find a campaign name")
    }


    select_campaign_window(app)
}