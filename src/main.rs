use druid::widget::prelude::*;
use druid::widget::{Flex, Label, Button};
use druid::{AppLauncher, Data, UnitPoint, WidgetExt, WindowDesc};

#[derive(Clone, Data)]
struct Campaign {
    name: String
}


fn choose_campaign_widget() -> impl Widget<Campaign>{
    let message = Label::new("Choose campaign:");
    let dummy_campaign = Button::new(|data: &Campaign, _env: &Env| data.name.to_string());

    Flex::column()
        .with_child(message)
        .with_spacer(20.0)
        .with_child(dummy_campaign)
        .align_vertical(UnitPoint::CENTER)

}
fn main() {
    //Describe main window
    let campaign_selection = WindowDesc::new(choose_campaign_widget())
        .title("Hello World!")
        .window_size((400.0, 400.0));

    let option: Campaign = Campaign { 
        name: "Uclia".into(),
     };
    //Start the application
    AppLauncher::with_window(campaign_selection)
        .log_to_console()
        .launch(option)
        .expect("Failed to start application");
}
