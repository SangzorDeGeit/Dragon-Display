use std::fs;
use std::{env, fs::OpenOptions, io::Read, sync::mpsc};

use async_channel::{Receiver, Sender};
use base64::{engine::general_purpose::STANDARD, Engine};
use google_drive::Client;
use gtk::glib::{self, spawn_future_local, subclass::prelude::*};
use gtk::glib::{clone, spawn_future};
use reqwest::blocking::Client as ReqwestClient;
use rouille::{Response, Server};
use snafu::{OptionExt, Report, ResultExt};

use tokio::fs::File;
use tokio::io::copy;

use crate::{
    errors::{
        AddressInUseSnafu, ClientSecretSnafu, ClientSnafu, ConnectionRefusedSnafu,
        DragonDisplayError, IOSnafu, InvalidDataSnafu, RecvSnafu, SendMessageSnafu,
    },
    setup::Token,
    try_emit,
    widgets::google_folder_object::GoogleFolderObject,
};

const SCOPE: &str = "https://www.googleapis.com/auth/drive.readonly";

pub enum GdClientEvent {
    Url {
        url: String,
    },
    Error {
        msg: String,
        fatal: bool,
    },
    Accesstoken {
        token: Token,
    },
    Totalfolders {
        total: usize,
    },
    Childrenfolders {
        parent: GoogleFolderObject,
        children: usize,
    },
    Folder {
        folder: GoogleFolderObject,
    },
    DownloadFile {
        id: String,
        name: String,
    },
    FileDownloaded,
    FailedFiles {
        files: Vec<String>,
    },
    Finished,
    Reconnect,
    Refresh,
}

mod imp {

    use std::sync::OnceLock;

    use super::*;

    #[derive(Default)]
    pub struct DragonDisplayGDClient {
        pub shutdown_receiver: OnceLock<async_channel::Receiver<()>>,
        pub event_sender: OnceLock<async_channel::Sender<GdClientEvent>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DragonDisplayGDClient {
        const NAME: &'static str = "DdGDClient";
        type Type = super::DragonDisplayGDClient;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for DragonDisplayGDClient {}
}

glib::wrapper! {
    pub struct DragonDisplayGDClient(ObjectSubclass<imp::DragonDisplayGDClient>);
}

impl DragonDisplayGDClient {
    /// Create a new instance of the initialize client object
    pub fn new(event_sender: Sender<GdClientEvent>) -> Self {
        let obj = glib::Object::new::<Self>();
        obj.imp()
            .event_sender
            .set(event_sender)
            .expect("Sender was already set");
        obj
    }

    /**
     * ----------------------------------------------------------------------
     *
     * Connect to google drive (create access and refresh token from nothing)
     *
     * ----------------------------------------------------------------------
     **/

    /// Connect to google drive. This function will block the main thread and must be called from a
    /// seperate runtime (using runtime().spawn())
    pub async fn connect(&self, shutdown_receiver: Receiver<()>) {
        try_emit!(self, Self::configure_environment(), true);
        let (tx, rx) = mpsc::channel();
        let (_, shutdown_sender) = try_emit!(self, self.start_server(tx), true);

        // shutdown the server if a shutdown signal is received
        spawn_future(
            clone!(@strong self as obj, @strong shutdown_receiver, @strong shutdown_sender => async move {
                while let Ok(_) = shutdown_receiver.recv().await {
                    try_emit!(
                        obj,
                        shutdown_sender.send(()).context(SendMessageSnafu),
                        false
                    );
                }
            }),
        );

        let mut google_drive_client = Client::new_from_env("", "").await;
        let user_consent_url = google_drive_client.user_consent_url(&[SCOPE.to_string()]);

        try_emit!(
            self,
            open::that(&user_consent_url).context(IOSnafu {
                msg: "Could not open a browser session".to_owned(),
            }),
            false
        );
        self.emit_event(GdClientEvent::Url {
            url: user_consent_url,
        });

        let state_and_code = try_emit!(
            self,
            rx.recv().context(RecvSnafu {
                msg: "Failed to receive message from listening server".to_owned()
            }),
            true
        );

        try_emit!(
            self,
            shutdown_sender.send(()).context(SendMessageSnafu),
            false
        );

        let access_token = try_emit!(
            self,
            google_drive_client
                .get_access_token(&state_and_code.1, &state_and_code.0)
                .await
                .context(ClientSnafu {
                    msg: "Could not get access token".to_owned(),
                }),
            true
        );
        let token = Token {
            access_token: access_token.access_token,
            refresh_token: access_token.refresh_token,
        };
        self.emit_event(GdClientEvent::Accesstoken { token });
    }

    /// Starts a server that listens for the google state and code, for connecting with google drive
    fn start_server(
        &self,
        tx: mpsc::Sender<(String, String)>,
    ) -> Result<(std::thread::JoinHandle<()>, std::sync::mpsc::Sender<()>), DragonDisplayError>
    {
        let obj = self.clone();
        let server = Server::new("localhost:8000", move |request| {
            match obj.get_state_and_code(request.raw_url()) {
                Ok(state_and_code) => {
                    tx.send(state_and_code).unwrap();
                    Response::text("linked succesfully, you can close this page now!")
                }
                Err(e) if matches!(e, DragonDisplayError::AddressInUse) => {
                    Response::text("sssshhhhh! I'm trying to listen!")
                }
                Err(_) => Response::text("The state or code given by google was invalid!"),
            }
        });

        let s = server.context(ConnectionRefusedSnafu {
            msg: "Could not start the listening server",
        })?;

        Ok(s.stoppable())
    }

    /// Extract the state and code from a google response
    fn get_state_and_code(&self, request: &str) -> Result<(String, String), DragonDisplayError> {
        let request_string = request.to_string();
        let state_stripped = request_string
            .strip_prefix("/?state=")
            .context(AddressInUseSnafu)?;

        let scope_stripped = state_stripped
            .strip_suffix(&format!("&scope={}", &SCOPE))
            .context(InvalidDataSnafu {
                msg: "The request gotten from google has unexpected format (no 'scope' found)",
            })?;

        let state_and_code = scope_stripped
            .rsplit_once("&code=")
            .context(InvalidDataSnafu {
                msg: "Could not find code in the google request",
            })?;

        let state_and_code = (state_and_code.0.to_owned(), state_and_code.1.to_owned());

        return Ok(state_and_code);
    }
    /**
     * ---------------------------------------------
     *
     * List folders in googledrive
     *
     * ---------------------------------------------
     **/
    /// Gets the total amount of folders in the targets google drive.
    pub async fn total_folders(&self, token: Token) {
        try_emit!(self, Self::configure_environment(), true);
        let query = format!("mimeType = 'application/vnd.google-apps.folder' and trashed = false");
        let google_drive_client =
            Client::new_from_env(token.access_token, token.refresh_token).await;
        let request = google_drive_client
            .files()
            .list_all(
                "user", "", false, "", false, "name", &query, "", false, false, "",
            )
            .await;
        let result = match request {
            Ok(r) => r.body,
            Err(_) => {
                self.emit_event(GdClientEvent::Refresh);
                return;
            }
        };
        let total = result.len();
        for file in result {
            let folder = GoogleFolderObject::new(file.id, file.name);
            self.emit_event(GdClientEvent::Folder { folder });
        }
        self.emit_event(GdClientEvent::Totalfolders { total })
    }

    /// Update the children variable of each GoogleFolderObject in the given `folders` Vec.
    pub async fn list_folders(&self, token: Token, mut folders: Vec<GoogleFolderObject>) {
        try_emit!(self, Self::configure_environment(), true);
        let google_drive_client =
            Client::new_from_env(&token.access_token, &token.refresh_token).await;

        while let Some(folder) = folders.pop() {
            let query = format!(
                "mimeType = 'application/vnd.google-apps.folder' and '{}' in parents and trashed = false",
                folder.id()
            );
            let request = google_drive_client
                .files()
                .list_all(
                    "user", "", false, "", false, "name", &query, "", false, false, "",
                )
                .await;

            let result = match request {
                Ok(r) => r.body,
                Err(_) => {
                    self.emit_event(GdClientEvent::Refresh);
                    return;
                }
            };

            let children: Vec<String> = result.into_iter().map(|file| file.id).collect();
            let amount = children.len();
            folder.set_children(children);
            self.emit_event(GdClientEvent::Childrenfolders {
                parent: folder,
                children: amount,
            });
        }
        self.emit_event(GdClientEvent::Finished);
        self.imp()
            .event_sender
            .get()
            .expect("Expected a sender")
            .close();
    }

    /**
     * ---------------------------------------------
     *
     * Synchronize folder from googledrive
     *
     * ---------------------------------------------
     **/

    /// Get the files in the google drive 'folder' that need to be downloaded, remove any local
    /// file that is not in the drive folder
    pub async fn get_and_remove(&self, token: Token, folder: String, path: String) {
        try_emit!(self, Self::configure_environment(), true);
        let google_drive_client =
            Client::new_from_env(&token.access_token, &token.refresh_token).await;
        // get existing local files
        let mut existing_files = Vec::new();
        let existing = try_emit!(
            self,
            fs::read_dir(&path).context(IOSnafu {
                msg: "Could not read directory".to_string()
            }),
            true
        );
        for f in existing {
            let f = match f {
                Ok(f) => f,
                Err(_) => continue,
            };
            let binding = f.file_name();
            let f = match binding.to_str() {
                Some(f) => f,
                None => continue,
            };
            existing_files.push(f.to_string());
        }
        // Get the files in the google drive folder
        let query = format!(
            "mimeType != 'application/vnd.google-apps.folder' and '{}' in parents and trashed = false",
            folder
        );
        let request = google_drive_client
            .files()
            .list_all(
                "user", "", false, "", false, "name", &query, "", false, false, "",
            )
            .await;
        let result = match request {
            Ok(r) => r.body,
            Err(_) => {
                self.emit_event(GdClientEvent::Refresh);
                return;
            }
        };
        let mut total = 0;
        // files that should not be removed
        let mut keep_files = Vec::new();
        for file in result {
            let mut keep = existing_files
                .iter()
                .filter(|f| *f == &file.name)
                .map(|f| f.clone())
                .collect();
            keep_files.append(&mut keep);
            if !existing_files.contains(&file.name) {
                self.emit_event(GdClientEvent::DownloadFile {
                    id: file.id,
                    name: file.name,
                });
                total += 1;
            }
        }
        for existing_file in existing_files {
            if !keep_files.contains(&&existing_file) {
                let _ = fs::remove_file(format!("{}/{}", &path, existing_file));
                continue;
            }
            total += 1;
        }
        self.emit_event(GdClientEvent::Totalfolders { total });
    }

    /// Download the vector of file ids from the google drive to the destination path
    pub async fn download_files(
        &self,
        token: Token,
        files: Vec<(String, String)>,
        destination: String,
    ) {
        let mut failed_files = Vec::new();
        for (id, name) in files {
            let download_url =
                format!("https://www.googleapis.com/drive/v3/files/{}?alt=media", id);
            let reqwest_client = ReqwestClient::new();
            let response = reqwest_client
                .get(&download_url)
                .bearer_auth(&token.access_token)
                .send();
            let result = match response {
                Ok(r) => r,
                Err(_) => {
                    self.emit_event(GdClientEvent::Refresh);
                    return;
                }
            };
            let mut destination = match File::create(format!("{}/{}", destination, name)).await {
                Ok(d) => d,
                Err(_) => {
                    failed_files.push(name);
                    continue;
                }
            };
            let bytes = match result.bytes() {
                Ok(r) => r,
                Err(_) => {
                    failed_files.push(name);
                    continue;
                }
            };
            if let Err(_) = copy(&mut bytes.as_ref(), &mut destination).await {
                failed_files.push(name);
                continue;
            }
            self.emit_event(GdClientEvent::FileDownloaded);
        }
        if !failed_files.is_empty() {
            self.emit_event(GdClientEvent::FailedFiles {
                files: failed_files,
            });
        }
        self.emit_event(GdClientEvent::Finished);
        self.imp().event_sender.get().unwrap().close();
    }

    /**
     * ---------------------------------------------
     *
     * Helper functions
     *
     * ---------------------------------------------
     **/

    /// Set the GOOGLE_KEY_ENCODED environment variable to enable calling client::new_from_env
    /// A file named client_secret.json needs to be in the directory of the Dragon-Display program
    fn configure_environment() -> Result<(), DragonDisplayError> {
        if let Ok(_) = env::var("GOOGLE_KEY_ENCODED") {
            return Ok(());
        }

        let mut path = env::current_dir().context(ClientSecretSnafu)?;
        path.push("client_secret.json");

        let mut file = OpenOptions::new()
            .read(true)
            .open(&path)
            .context(ClientSecretSnafu)?;

        // read the contents of the file to a string
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .context(ClientSecretSnafu)?;

        //the variable to add as environment variable (base64 encoded json string)
        let encoded_client_secret = STANDARD.encode(contents);

        //set the variable as GOOGLE_KEY_ENCODED
        env::set_var("GOOGLE_KEY_ENCODED", encoded_client_secret);
        Ok(())
    }

    /// takes in an old refresh and access token and returns a new one;
    pub async fn refresh_client(&self, token: Token) {
        let google_drive_client =
            Client::new_from_env(token.access_token, token.refresh_token).await;
        match google_drive_client.refresh_access_token().await {
            Ok(t) => self.emit_event(GdClientEvent::Accesstoken {
                token: Token {
                    access_token: t.access_token,
                    refresh_token: t.refresh_token,
                },
            }),
            Err(_) => self.emit_event(GdClientEvent::Reconnect),
        }
    }

    /**
     * ---------------------------------------------
     *
     * GObject Connections and Functions
     *
     * ---------------------------------------------
     **/

    /// Send a signal through the async_channel
    pub fn emit_event(&self, event: GdClientEvent) {
        let sender = self.imp().event_sender.get().expect("expected a sender");
        sender.send_blocking(event).expect("Failed to send message");
    }

    /// Emit an error message based on the input error
    pub fn emit_error(&self, err: DragonDisplayError, fatal: bool) {
        let msg = Report::from_error(err).to_string();
        self.emit_event(GdClientEvent::Error { msg, fatal });
    }

    /// Connect to an GdClientEvent -> an event that the client can sent
    pub fn connect_event<F: Fn(GdClientEvent) + 'static>(
        receiver: async_channel::Receiver<GdClientEvent>,
        f: F,
    ) {
        spawn_future_local(async move {
            while let Ok(event) = receiver.recv().await {
                f(event);
            }
        });
    }
}
