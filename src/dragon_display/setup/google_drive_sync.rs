use base64::{engine::general_purpose::STANDARD, Engine};
use google_drive::{AccessToken, Client};
use rouille::{Response, Server};
use std::{
    collections::HashMap, env, fs::OpenOptions, io::{self, Error, ErrorKind, Read}, sync::mpsc
};

use super::Campaign;
const SCOPE: &str = "https://www.googleapis.com/auth/drive.readonly";

pub enum InitializeMessage {
    UserConsentUrl { url: String },
    Token { token: AccessToken },
    Error { error: Error },
}

pub struct FolderResult<'a> {
    pub id_name_map: HashMap<String, String>,
    pub id_children_map: HashMap<&'a str, Vec<&'a str>>,
    pub access_token: String,
    pub refresh_token: String,
}

/// Initializes a google drive client using the oauth process
pub async fn initialize(sender: async_channel::Sender<InitializeMessage>) {
    match configure_environment() {
        Ok(_) => (),
        Err(e) => {
            sender.send_blocking(InitializeMessage::Error { error: e })
                .expect("Drive Frontend channel closed");
            return
        }
    }
    //initialize client
    let mut google_drive_client = Client::new_from_env("", "").await;

    //make a consent url
    let user_consent_url = google_drive_client.user_consent_url(&[SCOPE.to_string()]);

    let (tx, rx) = mpsc::channel();

    //start the target server for the redirect
    let (_handler, server_sender) = match start_server(tx) {
        Ok((h, s)) => (h, s),
        Err(e) => {
            sender.send_blocking(InitializeMessage::Error { error: e })
                .expect("Drive Frontend channel closed");
            return
        }
    };

    match open::that(user_consent_url.clone()) {
        Ok(_) => (),
        Err(e) => {
            sender.send_blocking(InitializeMessage::Error { error: e })
                .expect("Drive Frontend channel closed");
            return
        }
    }
    sender.send_blocking(InitializeMessage::UserConsentUrl { url: user_consent_url }).expect("Drive Frontend channel closed");

    println!("waiting for the message");
    //wait until the state and code vars are set
    let state_and_code = match rx.recv() {
        Ok(s) => s,
        Err(_) => {
            sender.send_blocking(InitializeMessage::Error { error: Error::new(ErrorKind::BrokenPipe, "Channel closed while listening on the server") })
                .expect("Drive Frontend channel closed");
            return
        }
    };
    println!("gotten the message");

    //tell the listening server to shut down
    match server_sender.send(()) {
        Ok(_) => (),
        Err(_) => {
            sender.send_blocking(InitializeMessage::Error { error: Error::new(ErrorKind::ConnectionAborted, "Could not close the listening server")})
                .expect("Drive Frontend channel closed");
            return
        }
    }
    println!("told server to shut down");
    
    match google_drive_client.get_access_token(&state_and_code.1, &state_and_code.0).await {
        Ok(result) => sender.send_blocking(InitializeMessage::Token { token: result }).expect("Drive Frontend channel closed"),
        Err(_) => sender.send_blocking(InitializeMessage::Error { error: Error::new(ErrorKind::Other, "Could not retrieve the access token from the google response")})
                .expect("Drive Frontend channel closed"),
    };
}

/// Starts a server that listens for the google state and code, for connecting with google drive
fn start_server(
    tx: mpsc::Sender<(String, String)>,
) -> Result<(std::thread::JoinHandle<()>, std::sync::mpsc::Sender<()>), io::Error> {
    let server = Server::new("localhost:8000", move |request| {
        match get_state_and_code(request.raw_url()) {
            Ok(state_and_code) => {
                tx.send(state_and_code).unwrap();
                Response::text("linked succesfully, you can close this page now!")
            }
            Err(e) => match e.kind() {
                ErrorKind::AddrInUse => Response::text("sssshhhhh! I'm trying to listen!"),
                _ => Response::text("The state or code given by google was invalid!"),
            },
        }
    });

    match server {
        Ok(s) => {
            return Ok(s.stoppable());
        }
        Err(_) => Err(Error::new(ErrorKind::ConnectionRefused, "Could not start the listening server")),
    }
}

/// Extracts the state and code from a google response
fn get_state_and_code(request: &str) -> Result<(String, String), io::Error> {
    let request_string = request.to_string(); let state_stripped = match request_string.strip_prefix("/?state=") { Some(s) => s, None => return Err(Error::new(ErrorKind::AddrInUse, "Could not find state and code in the request")),
    };

    let scope_stripped = match state_stripped.strip_suffix(&format!("&scope={}", &SCOPE)) {
        Some(s) => s,
        None => return Err(Error::new(ErrorKind::InvalidData, "The request gotten from google has unexpected format (no 'scope' found)")),
    };

    let state_and_code = match scope_stripped.rsplit_once("&code=") {
        Some(s) => (s.0.to_owned(), s.1.to_owned()),
        None => return Err(Error::new(ErrorKind::InvalidData, "Could not find code in the google request")),
    };
    return Ok(state_and_code);
}

/// Does a request to google to list all the folders under the given folder_id and returns all the
/// child folders and the tokens, in the case they are updated during the request
/// This method recursively calls itself to return two hashmaps. One linking each id to a vector of
/// children, the other linking each id to their name. This function should be called with
/// folder_id = 'root'.
pub async fn get_folder_tree<'a>(folder_result: &'a mut FolderResult<'a>, folder_id: String) -> Result<&'a mut FolderResult<'a>, io::Error> {

    configure_environment()?;

    // Extract all needed variables from the folder result
    let id_name_map = &mut folder_result.id_name_map;
    let id_children_map = &mut folder_result.id_children_map;
    let access_token = &mut folder_result.access_token;
    let refresh_token = &mut folder_result.refresh_token;

    //query to look for the 'folder' named 'Uclia' on the client drive
    let query = format!("mimeType = 'application/vnd.google-apps.folder' and '{}' in parents", folder_id);
    // Try the query maximum twice: if the first request does not get a response we try to
    // reconnect and try the query again. If it does not work a second time it will fail


    id_name_map.insert("hello".to_string(), "goodbye".to_string());
    for i in 0..2 {
        let mut google_drive_client = Client::new_from_env(&access_token, &refresh_token).await;
        google_drive_client.set_auto_access_token_refresh(true);
        println!("Requesting");
        let request = google_drive_client
                .files()
                .list_all("user", "", false, "", false, "name", &query, "", false, false, "").await;


        let response = match request {
            Ok(r) => r,
            Err(_) => {
                println!("response error!");
                if i==1 {
                    return Err(Error::new(ErrorKind::NotConnected, "Could not connect to google drive"));
                }
                (access_token, refresh_token) = refresh_client(access_token, refresh_token).await?;
                continue;
            }, 
        };
        let mut children: Vec<&str> = Vec::new();
        for folder in response.body {
            id_name_map.insert(folder.id.clone(), folder.name);
        }
        for id in id_name_map.keys() {
            children.push(&id);
        }
        id_children_map.insert(&folder_id, children);

        break;
    }

    return Ok(folder_result)
    
}

/// Downloads the files from google drive to the designated folder
pub fn sync_drive(campaign: Campaign) -> Result<(String, String), io::Error> {
    todo!();
}

/// Set the GOOGLE_KEY_ENCODED environment variable to enable calling client::new_from_env
/// A file named client_secret.json needs to be in the directory of the Dragon-Display program
fn configure_environment() -> Result<(), io::Error> {
    match env::var("GOOGLE_KEY_ENCODED") {
        Ok(_) => return Ok(()),
        Err(_) => (),
    };

    let mut path = env::current_dir()?;
    path.push("client_secret.json");

    let mut file = match OpenOptions::new().read(true).open(&path) {
        Ok(f) => f,
        Err(e) => {
            match e.kind() {
                ErrorKind::NotFound => return Err(Error::new(ErrorKind::NotFound, "Could not find client_secret.json file, please see the readme on github information on configuring google drive")),
                ErrorKind::PermissionDenied => return Err(Error::new(ErrorKind::PermissionDenied, "Could not get permission to read the client_secret.json")),
                _ => return Err(Error::new(ErrorKind::Other, "Some unknown error occured while trying to read the client_secret.json file")),
            }
        }
    };

    // read the contents of the file to a string
    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => (),
        Err(_) => return Err(Error::new(ErrorKind::InvalidData, "The client_secret.json file contained invalid data, please see the github readme for information on configuring google drive")),
    };

    //the variable to add as environment variable (base64 encoded json string)
    let encoded_client_secret = STANDARD.encode(contents);

    //set the variable as GOOGLE_KEY_ENCODED
    env::set_var("GOOGLE_KEY_ENCODED", encoded_client_secret);
    Ok(())
}

// takes in an old refresh and access token and returns a new one;
async fn refresh_client<'a>(access_token: &'a mut String, refresh_token: &'a mut String) -> Result<(&'a mut String, &'a mut String), Error> {
    let google_drive_client = Client::new_from_env(old_access_token, old_refresh_token).await;
    let token = google_drive_client.refresh_access_token().await;

    let token = match token {
        Ok(t) => t,
        Err(_) => todo!("There should be some re-initialization here"),
    };

    //TODO: The new tokens should be stored in the campaign config file
    let new_access_token = &mut token.access_token;
    let new_refresh_token = &mut token.refresh_token;
    old_access_token = new_access_token;
    old_refresh_token = new_refresh_token;
    return Ok((new_access_token, new_refresh_token));
}
