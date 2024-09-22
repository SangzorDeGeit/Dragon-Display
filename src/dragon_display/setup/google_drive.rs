// File containing functions for google drive synchronizing functionality
use async_recursion::async_recursion;
use base64::{engine::general_purpose::STANDARD, Engine};
use google_drive::{AccessToken, Client};
use rouille::{Response, Server};
use std::{
    collections::HashMap,
    env,
    fs::OpenOptions,
    io::{self, Error, ErrorKind, Read},
    sync::mpsc,
};

use gtk::glib::{clone, spawn_future};

use super::config::Campaign;
use super::ui::google_drive::FolderAmount;
const SCOPE: &str = "https://www.googleapis.com/auth/drive.readonly";

pub enum InitializeMessage {
    UserConsentUrl { url: String },
    Token { token: AccessToken },
    Error { error: Error },
}

#[derive(Clone)]
/// A data structure containing the maps used to create the googledrive folder tree and the up-to-date access and refresh token
pub struct FolderResult {
    // hashmap linking folder id to folder name
    pub id_name_map: HashMap<String, String>,
    // hashmap linking folder id to children ids
    pub id_child_map: HashMap<String, Vec<String>>,
    pub access_token: String,
    pub refresh_token: String,
}

/// Initializes a google drive client using the oauth process
pub async fn initialize_client(
    sender: async_channel::Sender<InitializeMessage>,
    server_terminator: async_channel::Receiver<()>,
) {
    match configure_environment() {
        Ok(_) => (),
        Err(e) => {
            sender
                .send_blocking(InitializeMessage::Error { error: e })
                .expect("Drive Frontend channel closed");
            return;
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
            sender
                .send_blocking(InitializeMessage::Error { error: e })
                .expect("Drive Frontend channel closed");
            return;
        }
    };

    // await for a possible message to terminate the server
    // early return if the server should be terminated early
    spawn_future(clone!(@strong server_sender, @strong sender => async move {
        while let Ok(_) = server_terminator.recv().await {
            match server_sender.send(()) {
                Ok(_) => return,
                Err(_) => {
                    sender
                        .send_blocking(InitializeMessage::Error {
                            error: Error::new(
                                       ErrorKind::ConnectionAborted,
                                       "Could not close the listening server",
                                   ),
                        })
                    .expect("Drive Frontend channel closed");
                    return;
                }
            }
        }
    }));

    match open::that(user_consent_url.clone()) {
        Ok(_) => (),
        Err(e) => {
            sender
                .send_blocking(InitializeMessage::Error { error: e })
                .expect("Drive Frontend channel closed");
            return;
        }
    }
    sender
        .send_blocking(InitializeMessage::UserConsentUrl {
            url: user_consent_url,
        })
        .expect("Drive Frontend channel closed");

    //wait until the state and code vars are set
    let state_and_code = match rx.recv() {
        Ok(s) => s,
        Err(_) => {
            sender
                .send_blocking(InitializeMessage::Error {
                    error: Error::new(
                        ErrorKind::BrokenPipe,
                        "Channel closed while listening on the server",
                    ),
                })
                .expect("Drive Frontend channel closed");
            return;
        }
    };

    //tell the listening server to shut down
    match server_sender.send(()) {
        Ok(_) => (),
        Err(_) => {
            sender
                .send_blocking(InitializeMessage::Error {
                    error: Error::new(
                        ErrorKind::ConnectionAborted,
                        "Could not close the listening server",
                    ),
                })
                .expect("Drive Frontend channel closed");
            return;
        }
    }

    match google_drive_client
        .get_access_token(&state_and_code.1, &state_and_code.0)
        .await
    {
        Ok(result) => sender
            .send_blocking(InitializeMessage::Token { token: result })
            .expect("Drive Frontend channel closed"),
        Err(_) => sender
            .send_blocking(InitializeMessage::Error {
                error: Error::new(
                    ErrorKind::Other,
                    "Could not retrieve the access token from the google response",
                ),
            })
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
        Err(_) => Err(Error::new(
            ErrorKind::ConnectionRefused,
            "Could not start the listening server",
        )),
    }
}

/// Tries to extract the state and code from a google response
fn get_state_and_code(request: &str) -> Result<(String, String), io::Error> {
    let request_string = request.to_string();
    let state_stripped = match request_string.strip_prefix("/?state=") {
        Some(s) => s,
        None => {
            return Err(Error::new(
                ErrorKind::AddrInUse,
                "Could not find state and code in the request",
            ))
        }
    };

    let scope_stripped = match state_stripped.strip_suffix(&format!("&scope={}", &SCOPE)) {
        Some(s) => s,
        None => {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "The request gotten from google has unexpected format (no 'scope' found)",
            ))
        }
    };

    let state_and_code = match scope_stripped.rsplit_once("&code=") {
        Some(s) => (s.0.to_owned(), s.1.to_owned()),
        None => {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Could not find code in the google request",
            ))
        }
    };
    return Ok(state_and_code);
}

/// Does a recursive request to google to create a folder tree of the target googledrive. Each
/// iteration looks up the child folders for the input folder_id and calls this function on each of
/// those folders.
#[async_recursion]
pub async fn get_folder_tree(
    folder_result: FolderResult,
    folder_id: String,
    sender: async_channel::Sender<FolderAmount>,
) -> Result<FolderResult, io::Error> {
    // The amount of child folders under the current id
    let mut child_folders: usize = 0;

    configure_environment()?;
    let mut access_token = folder_result.access_token.clone();
    let mut refresh_token = folder_result.refresh_token.clone();

    let query = format!(
        "mimeType = 'application/vnd.google-apps.folder' and '{}' in parents and trashed = false",
        folder_id
    );
    // Try the query maximum twice: if the first request does not get a response we try to
    // reconnect and try the query again. If it does not work a second time it will fail
    for i in 0..2 {
        let mut google_drive_client = Client::new_from_env(&access_token, &refresh_token).await;
        google_drive_client.set_auto_access_token_refresh(true);
        let request = google_drive_client
            .files()
            .list_all(
                "user", "", false, "", false, "name", &query, "", false, false, "",
            )
            .await;

        let response = match request {
            Ok(r) => r,
            Err(_) => {
                if i == 0 {
                    (access_token, refresh_token) =
                        refresh_client(&access_token, &refresh_token).await?;
                    continue;
                } else {
                    break;
                }
            }
        };

        let mut id_name_map = folder_result.id_name_map.clone();
        let mut children: Vec<String> = Vec::new();
        for file in response.body {
            child_folders += 1;
            id_name_map.insert(file.id.clone(), file.name);
            children.push(file.id)
        }
        //The amount of child folders is send to the progress bar
        sender
            .send_blocking(FolderAmount::Current {
                amount: child_folders,
            })
            .expect("channel closed");

        let mut id_child_map = folder_result.id_child_map.clone();
        id_child_map.insert(folder_id.clone(), children.clone());

        let mut folder_result = FolderResult {
            id_name_map,
            id_child_map,
            access_token,
            refresh_token,
        };
        for child in children {
            folder_result = get_folder_tree(folder_result, child, sender.clone()).await?;
        }
        return Ok(folder_result);
    }
    return Err(Error::new(
        ErrorKind::NotConnected,
        "Could not connect to google drive",
    ));
}

/// Gets the total amount of folders in the targets google drive.
pub async fn get_folder_amount(
    access_token: String,
    refresh_token: String,
) -> Result<(usize, String, String), Error> {
    configure_environment()?;
    let mut access_token = access_token;
    let mut refresh_token = refresh_token;

    let query = format!("mimeType = 'application/vnd.google-apps.folder' and trashed = false");
    // Try the query maximum twice: if the first request does not get a response we try to
    // reconnect and try the query again. If it does not work a second time it will fail
    for i in 0..2 {
        let mut google_drive_client = Client::new_from_env(&access_token, &refresh_token).await;
        google_drive_client.set_auto_access_token_refresh(true);
        let request = google_drive_client
            .files()
            .list_all(
                "user", "", false, "", false, "name", &query, "", false, false, "",
            )
            .await;

        let response = match request {
            Ok(r) => r,
            Err(_) => {
                if i == 0 {
                    (access_token, refresh_token) =
                        refresh_client(&access_token, &refresh_token).await?;
                    continue;
                } else {
                    break;
                }
            }
        };

        return Ok((response.body.len(), access_token, refresh_token));
    }
    return Err(Error::new(
        ErrorKind::NotConnected,
        "Could not connect to google drive",
    ));
}

/// Downloads the files from google drive to the designated folder
pub fn sync_drive(campaign: Campaign) -> Result<(String, String), io::Error> {
    todo!();
}

/// Set the GOOGLE_KEY_ENCODED environment variable to enable calling client::new_from_env
/// A file named client_secret.json needs to be in the directory of the Dragon-Display program
fn configure_environment() -> Result<(), io::Error> {
    if let Ok(_) = env::var("GOOGLE_KEY_ENCODED") {
        return Ok(());
    }

    let mut path = env::current_dir()?;
    path.push("client_secret.json");

    let mut file = match OpenOptions::new().read(true).open(&path) {
        Ok(f) => f,
        Err(e) => {
            match e.kind() {
                ErrorKind::NotFound => return Err(Error::new(ErrorKind::NotFound, "Could not find client_secret.json file, please see the github readme for information on configuring google drive")),
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
async fn refresh_client(
    access_token: &str,
    refresh_token: &str,
) -> Result<(String, String), Error> {
    let google_drive_client = Client::new_from_env(access_token, refresh_token).await;
    let token = google_drive_client.refresh_access_token().await;

    let token = match token {
        Ok(t) => t,
        Err(_) => todo!("There should be some re-initialization here"),
    };

    return Ok((token.access_token, token.refresh_token));
}
