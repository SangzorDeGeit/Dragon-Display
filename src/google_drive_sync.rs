#![feature(proc_macro_hygiene, decl_macro)]

use std::fmt::format;

use google_drive::Client;
use open;
#[macro_use] use rocket::*;
use rocket::http::uri::Origin;

const SCOPE: &str = "https://www.googleapis.com/auth/drive.readonly";

#[get("/")]
fn echo_fn(uri: &Origin) -> String {
    match uri.to_string().strip_prefix("/?state=") {
        Some(value) => {
            match value.to_string().strip_suffix(&format!("&scope={}",&SCOPE)) {
                Some(state_and_code) => {
                    if let Some(state_code) = state_and_code.rsplit_once("&code=") {
                        let state = state_code.0;
                        let code = state_code.1;
                        return format!("Successfully linked, can close this page now!\nstate = {}\ncode = {}",state.to_string(),code.to_string())
                    }
                },
                None => panic!("Got wrong return from google api: redirect did not end with scope")
            }
        },
        None => panic!("Got wrong return from google api: redirect did not start with /?state="),
    }
    return format!("succesfully linked!")
}


pub async fn initialize() {
    println!("Starting...");
    let mut google_drive_client = Client::new(
        "1043613452788-2rq3ksqhaivjtt5hjjp5o49a0n87nbh2.apps.googleusercontent.com",
        "",
        "http://localhost:8000/",
        "",
        ""
    );
        
    println!("made client");
    let user_consent_url = google_drive_client.user_consent_url(&[SCOPE.to_string()]);
    println!("The consent url: {}", user_consent_url);
    open::that(user_consent_url).expect("could not open page");
    start_listening().await;

    // let mut access_token = google_drive_client.get_access_token(code, state).await.unwrap();
    

}

pub async fn start_listening() {
    rocket::ignite().mount("/", routes![echo_fn]).launch();
}

