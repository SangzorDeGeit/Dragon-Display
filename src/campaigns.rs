use gtk::{prelude::*, ApplicationWindow};
use gtk::{Application, Button, Stack, Label, Box, glib};

// The 'add campaign' window
fn add_campaign_window(app: &Application) {
    let container = Box::new(gtk::Orientation::Vertical, 100);

    let stack = Stack::new();
    stack.set_transition_type(gtk::StackTransitionType::SlideLeftRight);
    stack.set_transition_duration(200);

    //setup page 1
    let page_1_label = Label::new(Some("Name the campaign"));
    let button_next = Button::builder()
        .label("Next")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();
    let page_1 = Box::new(gtk::Orientation::Vertical, 50);
    page_1.append(&page_1_label);
    page_1.append(&button_next);

    //setup page 2
    let page_2_label = Label::new(Some("Choose synchronization"));
    let button_prev = Button::builder()
        .label("Back")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();
    let page_2 = Box::new(gtk::Orientation::Vertical, 50);
    page_2.append(&page_2_label);
    page_2.append(&button_prev);

    // add pages as to stack
    stack.add_titled(&page_1, Option::<&str>::None, "Naming");
    stack.add_titled(&page_2, Option::<&str>::None, "Synchronization");

    container.append(&stack);

    //specify actions
    button_next.connect_clicked(glib::clone!(@strong stack => move |_| stack.set_visible_child(&page_2)));
    button_prev.connect_clicked(move |_| stack.set_visible_child(&page_1));


}

// The 'remove campaign' window
fn remove_campaign_window(){
    todo!();
}

// The "main"/"select campaign" window
pub fn select_campaign_window(app: &Application){
    //read config -> list of campaigns

    // Make the widget elements
    let add_button = Button::builder()
        .label("add campaign")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    let remove_button = Button::builder()
        .label("remove campaign")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    //make amount a button for each campaign

    //Make a box to put all the buttons in
    let container = Box::new(gtk::Orientation::Horizontal, 100);
    container.append(&add_button);
    container.append(&remove_button);

    // Connect the widgets to actions
    add_button.connect_clicked(move |_| add_campaign_window(app));
    // Make a stack containing the buttons

    //connect each button to sync the correct pictures and run the program


    //build the window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Dragon-Display")
        .child(&container)
        .build();


    window.present();
}
