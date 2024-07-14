use base64::{engine::general_purpose::STANDARD, Engine};
use futures::executor::block_on;
use google_drive::{AccessToken, Client};
use rouille::{Response, Server};
use std::{
    env,
    fs::OpenOptions,
    io::{self, Error, ErrorKind, Read},
    sync::mpsc,
};

const SCOPE: &str = "https://www.googleapis.com/auth/drive.readonly";

pub enum InitializeMessage {
    UserConsentUrl { url: String },
    Token { token: AccessToken },
    Error { error: Error },
}

/// Initializes a google drive client using the oauth process
pub fn initialize(sender: async_channel::Sender<InitializeMessage>) {
    match configure_environment() {
        Ok(_) => (),
        Err(e) => {
            sender.send_blocking(InitializeMessage::Error { error: e })
                .expect("Drive Frontend channel closed");
            return
        }
    }
    //initialize client
    let mut google_drive_client = block_on(Client::new_from_env("", ""));

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

    match open::that(user_consent_url) {
        Ok(_) => (),
        Err(e) => {
            sender.send_blocking(InitializeMessage::Error { error: e })
                .expect("Drive Frontend channel closed");
            return
        }
    }
    sender.send_blocking(InitializeMessage::UserConsentUrl { url: user_consent_url }).expect("Drive Frontend channel closed");

    //wait until the state and code vars are set
    let state_and_code = match rx.recv() {
        Ok(s) => s,
        Err(e) => {
            sender.send_blocking(InitializeMessage::Error { error: Error::new(ErrorKind::BrokenPipe, "Channel closed while listening on the server") })
                .expect("Drive Frontend channel closed");
            return
        }
    };

    //tell the listening server to shut down
    match server_sender.send(()) {
        Ok(_) => (),
        Err(e) => {
            sender.send_blocking(InitializeMessage::Error { error: Error::new(ErrorKind::ConnectionAborted, "Could not close the listening server")})
                .expect("Drive Frontend channel closed");
            return
        }
    }
    
    let result = match block_on(google_drive_client.get_access_token(&state_and_code.1, &state_and_code.0)) {
        Ok(s) => s,
        Err(e) => {
            sender.send_blocking(InitializeMessage::Error { error: Error::new(ErrorKind::Other, "Could not retrieve the access token from the google response")})
                .expect("Drive Frontend channel closed");
            return
        }
    };
    
    sender.send_blocking(InitializeMessage::Token { token: result }).expect("Drive Frontend channel closed");
        
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

/// This function makes a gui where the user can select the path in google drive for where they
/// want to download their files from
pub async fn select_path(campaign: Campaign) -> Result<(String, String), io::Error> {
    let (mut accesstoken, mut refresh_token) = match campaign.get_google_drive_properties() {
        Some(t) => (t.0, t.1),
        None => return Err(Error::new(ErrorKind::InvalidInput, "Sync was called on a no-sync campaign")),
    };

    configure_environment()?;

    //query to look for the 'folder' named 'Uclia' on the client drive
    let query_1 = "mimeType = 'application/vnd.google-apps.folder' and '1xG9_N833F2qqDQ5NaDA7G7OrR4UMnJyz' in parents";
    // Try the query maximum twice: if the first request does not get a response we try to
    // reconnect and try the query again. If it does not work a second time it will fail
    for i in 0..2 {
        let mut google_drive_client = Client::new_from_env(&accesstoken, &refresh_token).await;
        google_drive_client.set_auto_access_token_refresh(true);
        println!("Requesting");
        let folder = google_drive_client
                .files()
                .list_all("user", "", false, "", false, "name", query_1, "", false, false, "").await;


        let response = match folder {
            Ok(r) => r,
            Err(_) => {
                println!("response error!");
                if i==1 {
                    return Err(Error::new(ErrorKind::NotConnected, "Could not connect to google drive"));
                }
                (accesstoken, refresh_token) = refresh_client(&accesstoken, &refresh_token).await?;
                continue;
            }, 
        };
        for file in response.body {
            println!("name: {}, id: {}", file.name, file.id); 
        }
        break;
    }

    return Ok((String::from(accesstoken), String::from(refresh_token)));
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
async fn refresh_client(old_access_token: &str, old_refresh_token: &str) -> Result<(String, String), Error> {
    let google_drive_client = Client::new_from_env(old_access_token, old_refresh_token).await;
    let token = google_drive_client.refresh_access_token().await;

    let token = match token {
        Ok(t) => t,
        Err(_) => todo!("There should be some re-initialization here"),
    };

    //TODO: The new tokens should be stored in the campaign config file
    let new_access_token = token.access_token;
    let new_refresh_token = token.refresh_token;
    return Ok((new_access_token, new_refresh_token));
}
