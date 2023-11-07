use std::env;
use rouille::{Server, Response};
use google_drive::{Client, AccessToken, files, drives};
use open;


const SCOPE: &str = "https://www.googleapis.com/auth/drive.readonly";


pub async fn initialize() -> AccessToken {
    let mut google_drive_client = Client::new(
        "1043613452788-ij2c5k1k19jf4rqf8o0fg2hh0t71kvct.apps.googleusercontent.com",
        "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",
        "http://localhost:8000/",
        "",
        ""
    );

    let user_consent_url = google_drive_client.user_consent_url(&[SCOPE.to_string()]);
    println!("The consent url: {}", user_consent_url);

    let (_handler, sender) = start_server();
    open::that(user_consent_url).expect("could not open page");

    while let Err(_) = env::var("DRAGON_DISPLAY_CODE") {
    }
    sender.send(()).unwrap();
    
    let code_state = (env::var("DRAGON_DISPLAY_CODE"), env::var("DRAGON_DISPLAY_STATE"));

    match code_state {
        (Ok(code), Ok(state)) => {
            let access_token = google_drive_client.get_access_token(&code, &state).await.unwrap();
            return access_token
        },
        _ => todo!(),
    }
}

fn start_server() -> (std::thread::JoinHandle<()>, std::sync::mpsc::Sender<()>) {
    let server = Server::new("localhost:8000", move |request|{
        match request.raw_url().to_string().strip_prefix("/?state=") {
            Some(value) => {
                set_state_and_code(value);
                Response::text("linked succesfully, you can close this page now!")
            },
            None => Response::text("sssshhhhh! I'm trying to listen!"),
        }       
    });

    match server {
        Ok(s) => {
            return s.stoppable();
        },
        Err(_) => todo!()
    }
}

fn set_state_and_code(value: &str) {
    match value.to_string().strip_suffix(&format!("&scope={}",&SCOPE)) {
        Some(state_and_code) => {
            if let Some(state_code) = state_and_code.rsplit_once("&code=") {
                env::set_var("DRAGON_DISPLAY_STATE", state_code.0);
                env::set_var("DRAGON_DISPLAY_CODE", state_code.1);
            } else {
                todo!()
            }
        },
        None => todo!()
    }
}

