use std::{env,io::{self, Error, ErrorKind}};
use rouille::{Server, Response};
use google_drive::{Client, AccessToken};
use open;
use futures;
use tokio::runtime::Handle;


const SCOPE: &str = "https://www.googleapis.com/auth/drive.readonly";



pub fn initialize() -> Result<AccessToken, io::Error> {
    let mut google_drive_client = Client::new(
        "1043613452788-ij2c5k1k19jf4rqf8o0fg2hh0t71kvct.apps.googleusercontent.com",
        "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",
        "http://localhost:8000/",
        "",
        ""
    );

    let user_consent_url = google_drive_client.user_consent_url(&[SCOPE.to_string()]);
    println!("The consent url: {}", user_consent_url);

    let (_handler, sender) = start_server()?;
    open::that(user_consent_url).expect("could not open page");

    while let Err(_) = env::var("DRAGON_DISPLAY_CODE") {}
    sender.send(()).unwrap();
    
    let code_state = (env::var("DRAGON_DISPLAY_CODE"), env::var("DRAGON_DISPLAY_STATE"));


    match code_state {
        (Ok(code), Ok(state)) => {
            let handle = Handle::current();
            let _ = handle.enter();
            let access_token = futures::executor::block_on(async move {google_drive_client.get_access_token(&code, &state).await.unwrap()});
            return Ok(access_token)
        },
        _ => Err(Error::from(ErrorKind::WriteZero)),
    }
}




fn start_server() -> Result<(std::thread::JoinHandle<()>, std::sync::mpsc::Sender<()>), io::Error> {
    let server = Server::new("localhost:8000", move |request|{
        match request.raw_url().to_string().strip_prefix("/?state=") {
            Some(value) => {
                match set_state_and_code(value) {
                    Ok(_) => Response::text("linked succesfully, you can close this page now!"),
                    Err(_) => Response::text("The state or code given by google was invalid!")
                }
            },
            None => Response::text("sssshhhhh! I'm trying to listen!"),
        }       
    });

    match server {
        Ok(s) => {
            return Ok(s.stoppable());
        },
        Err(_) => Err(Error::from(ErrorKind::Other))
    }
}



fn set_state_and_code(value: &str) -> Result<(), io::Error>{
    match value.to_string().strip_suffix(&format!("&scope={}",&SCOPE)) {
        Some(state_and_code) => {
            if let Some(state_code) = state_and_code.rsplit_once("&code=") {
                env::set_var("DRAGON_DISPLAY_STATE", state_code.0);
                env::set_var("DRAGON_DISPLAY_CODE", state_code.1);
                Ok(())
            } else {
                return Err(Error::from(ErrorKind::InvalidData))
            }
        },
        None => return Err(Error::from(ErrorKind::InvalidData))
    }
}

