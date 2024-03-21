use std::{io::Read, env, fs::OpenOptions, io::{self, Error, ErrorKind}};
use rouille::{Server, Response};
use google_drive::{drives::Drives, files::Files, types::Drive, AccessToken, Client};
use open;
use futures::executor::block_on;
use base64::{engine::general_purpose::STANDARD, Engine};



const SCOPE: &str = "https://www.googleapis.com/auth/drive.readonly";


pub fn initialize() -> Result<AccessToken, io::Error> {
    configure_environment()?;
    //initialize client
    let mut google_drive_client = block_on(Client::new_from_env("", ""));

    //make a consent url
    let user_consent_url = google_drive_client.user_consent_url(&[SCOPE.to_string()]);

    //start the target server for the redirect
    let (_handler, sender) = start_server()?;
    open::that(user_consent_url).expect("could not open page");

    //wait until the state and code vars are set
    while let Err(_) = env::var("DRAGON_DISPLAY_CODE") {}

    //tell the listening server to shut down
    sender.send(()).unwrap();
    
    let code_state = (env::var("DRAGON_DISPLAY_CODE"), env::var("DRAGON_DISPLAY_STATE"));


    match code_state {
        (Ok(code), Ok(state)) => {
            //block the curren thread to generate an access token with the async function
            let access_token = block_on(google_drive_client.get_access_token(&code, &state)).unwrap();
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

/**
 * Set the GOOGLE_KEY_ENCODED environment variable to enable calling client::new_from_env
 * A file named client_secret.json needs to be in the directory of the Dragon-Display program
 */
fn configure_environment() -> Result<(), io::Error> {

    match env::var("GOOGLE_KEY_ENCODED"){
        Ok(_) => return Ok(()),
        Err(_) => (),
    };

    // Open the file that contains the client secret
    let mut path = env::current_dir()?;
    path.push("client_secret.json");

    let mut file = OpenOptions::new()
        .read(true)
        .open(&path)?;

    // read the contents of the file to a string
    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => (),
        Err(_) => return Err(Error::from(ErrorKind::Unsupported))
    };

    //the variable to add as environment variable (base64 encoded json string)
    let encoded_client_secret = STANDARD.encode(contents);

    //set the variable as GOOGLE_KEY_ENCODED
    env::set_var("GOOGLE_KEY_ENCODED", encoded_client_secret);
    Ok(())
}

/**
 * Synchronizes images of the target drive
 */
pub fn sync_drive(accesstoken: &str, refresh_token: &str){
    let _ = configure_environment();

    let (accesstoken, refresh_token) = refresh_client(accesstoken, refresh_token);

    let mut google_drive_client = block_on(Client::new_from_env(accesstoken, refresh_token));
    google_drive_client.set_auto_access_token_refresh(true);

    //query to look for the 'folder' named 'Uclia' on the client drive
    let query_1 = "name='Uclia' and mimeType = 'application/vnd.google-apps.folder'";
    let folder = block_on(google_drive_client.files().list_all("", "", false, "", false, "", query_1, "", false, false, ""));

    let response = match folder {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: {}", e);
            return;
        }
    };

    
    //Loop through all files in the response -> temporary
    for f in response.body {
        println!("file name: {}, file id: {}",f.name, f.id);
        let filter_query = format!("'{}' in parents and not trashed", f.id);
        let list = block_on(google_drive_client.files().list_all("", "", false, "", false, "", &filter_query, "", false, false, ""));

        let response_2 = match list {
            Ok(r) => r,
            Err(e) => {
                eprintln!("error: {}", e);
                return;
            }
        };

        for img in response_2.body {
            println!("file name: {}", img.name)
        }

    }   
    
}

fn refresh_client(old_access_token: &str, old_refresh_token: &str) -> (String, String) {
    let google_drive_client = block_on(Client::new_from_env(old_access_token, old_refresh_token));
    let token = block_on(google_drive_client.refresh_access_token());

    let token = match token {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: {}", e);
            return ("".to_string(), "".to_string());
        }
    };
    //TODO: The new tokens should be stored in the campaign config file
    let new_access_token = token.access_token;
    let new_refresh_token = token.refresh_token;
    return (new_access_token, new_refresh_token)
}
