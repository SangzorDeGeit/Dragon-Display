use std::{env, fs::OpenOptions, io::{self, Error, ErrorKind, Read}, sync::mpsc::{self, Sender}};
use rouille::{Server, Response};
use google_drive::{drives::Drives, files::Files, types::Drive, AccessToken, Client};
use open;
use futures::{channel::oneshot::channel, executor::block_on};
use base64::{engine::general_purpose::STANDARD, Engine};



const SCOPE: &str = "https://www.googleapis.com/auth/drive.readonly";


pub fn initialize() -> Result<AccessToken, Error> {
    configure_environment()?;
    //initialize client
    let mut google_drive_client = block_on(Client::new_from_env("", ""));

    //make a consent url
    let user_consent_url = google_drive_client.user_consent_url(&[SCOPE.to_string()]);

    let (tx, rx) = mpsc::channel();

    //start the target server for the redirect
    let (handler, _sender) = start_server(tx)?;
    open::that(user_consent_url).expect("Could not open the user consent url");

    //wait until the state and code vars are set
    let state_and_code = match rx.recv() {
        Ok(s) => s,
        Err(_) => return Err(Error::from(ErrorKind::BrokenPipe))
    }; 

    //tell the listening server to shut down
    handler.join().unwrap();


    return Ok(block_on(google_drive_client.get_access_token(&state_and_code.1, &state_and_code.0)).unwrap());
}




fn start_server(tx: mpsc::Sender<(String, String)>) -> Result<(std::thread::JoinHandle<()>, std::sync::mpsc::Sender<()>), io::Error> {
    let server = Server::new("localhost:8000", move |request|{
        match get_state_and_code(request.raw_url()) {
            Ok(state_and_code) => {
                tx.send(state_and_code).unwrap();
                Response::text("linked succesfully, you can close this page now!")
            },
            Err(e) => {
                match e.kind() {
                    ErrorKind::AddrInUse => Response::text("sssshhhhh! I'm trying to listen!"),
                    _ => Response::text("The state or code given by google was invalid!")
                }
                
            }
        }
    });

    match server {
        Ok(s) => {
            return Ok(s.stoppable());
        },
        Err(_) => Err(Error::from(ErrorKind::Other))
    }
}


/// Extracts the state and code from a google response
fn get_state_and_code(request: &str) -> Result<(String, String), io::Error>{
    let request_string = request.to_string();
    let state_stripped = match request_string.strip_prefix("/?state=") {
        Some(s) => s,
        None => return Err(Error::from(ErrorKind::AddrInUse)),
    };

    let scope_stripped = match state_stripped.strip_suffix(&format!("&scope={}",&SCOPE)) {
        Some(s) => s,
        None => return Err(Error::from(ErrorKind::InvalidData)),
    };


    let state_and_code = match scope_stripped.rsplit_once("&code=") {
        Some(s) => (s.0.to_owned(), s.1.to_owned()),
        None => return Err(Error::from(ErrorKind::InvalidData))
    };    
   return Ok(state_and_code);
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
