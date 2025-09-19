use std::{env, fs::OpenOptions, io::Read, sync::mpsc};

use base64::{engine::general_purpose::STANDARD, Engine};
use google_drive::Client;
use gtk::glib::{self, object::ObjectExt, subclass::prelude::*};
use rouille::{Response, Server};
use snafu::{OptionExt, Report, ResultExt};

use crate::{
    errors::{
        AddressInUseSnafu, ClientSecretSnafu, ClientSnafu, ConnectionRefusedSnafu,
        DragonDisplayError, IOSnafu, InvalidDataSnafu, OtherSnafu, RecvSnafu, SendMessageSnafu,
    },
    setup::Token,
    try_emit,
};

const SCOPE: &str = "https://www.googleapis.com/auth/drive.readonly";

mod imp {

    use std::sync::{mpsc::Sender, OnceLock};

    use gtk::glib::{subclass::Signal, types::StaticType};

    use super::*;

    #[derive(Default)]
    pub struct DragonDisplayGDClient {
        pub shutdown_sender: OnceLock<Sender<()>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for DragonDisplayGDClient {
        const NAME: &'static str = "DdGDClient";
        type Type = super::DragonDisplayGDClient;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for DragonDisplayGDClient {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
            SIGNALS.get_or_init(|| {
                vec![
                    Signal::builder("url")
                        .param_types([String::static_type()])
                        .build(),
                    Signal::builder("accesstoken")
                        .param_types([String::static_type(), String::static_type()])
                        .build(),
                    Signal::builder("total-folders")
                        .param_types([u32::static_type()])
                        .build(),
                    Signal::builder("reconnect").build(),
                    Signal::builder("refresh-total").build(),
                    Signal::builder("error")
                        .param_types([String::static_type(), bool::static_type()])
                        .build(),
                ]
            })
        }
    }
}

glib::wrapper! {
    pub struct DragonDisplayGDClient(ObjectSubclass<imp::DragonDisplayGDClient>);
}

impl DragonDisplayGDClient {
    /// Create a new instance of the initialize client object
    pub fn new() -> Self {
        let obj = glib::Object::new::<Self>();
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
    pub async fn connect(&self) {
        try_emit!(self, Self::configure_environment(), true);

        let (tx, rx) = mpsc::channel();
        let (_, shutdown_sender) = try_emit!(self, self.start_server(tx), true);
        self.imp()
            .shutdown_sender
            .set(shutdown_sender)
            .expect("Expected oncelock to be empty");

        let mut google_drive_client = Client::new_from_env("", "").await;
        let user_consent_url = google_drive_client.user_consent_url(&[SCOPE.to_string()]);

        self.emit_by_name::<()>("url", &[&user_consent_url]);
        try_emit!(
            self,
            open::that(&user_consent_url).context(IOSnafu {
                msg: "Could not open a browser session".to_owned(),
            }),
            false
        );

        let state_and_code = try_emit!(
            self,
            rx.recv().context(RecvSnafu {
                msg: "Failed to receive message from listening server".to_owned()
            }),
            true
        );

        try_emit!(self, self.shutdown_server(), true);

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

        self.emit_by_name::<()>(
            "accesstoken",
            &[&access_token.access_token, &access_token.refresh_token],
        );
    }

    /// Shutdown the server that listens for a google state and code, returns an error if no server
    /// was open
    pub fn shutdown_server(&self) -> Result<(), DragonDisplayError> {
        let shutdown_sender = self.imp().shutdown_sender.get().context(OtherSnafu {
            msg: "No server running".to_owned(),
        })?;
        shutdown_sender.send(()).context(SendMessageSnafu)?;

        Ok(())
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
        match request {
            Ok(r) => self.emit_by_name::<()>("total-folders", &[&(r.body.len() as u32)]),
            Err(_) => self.emit_by_name::<()>("refresh-total", &[]),
        }
    }

    pub async fn list_folders(&self, token: Token) {
        todo!("implement")
    }

    /**
     * ---------------------------------------------
     *
     * Synchronize folder from googledrive
     *
     * ---------------------------------------------
     **/

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
            .context(ClientSecretSnafu);

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
            Ok(t) => self.emit_by_name::<()>("accesstoken", &[&t.access_token, &t.refresh_token]),
            Err(_) => self.emit_by_name::<()>("reconnect", &[]),
        }
    }

    /**
     * ---------------------------------------------
     *
     * GObject Connections and Functions
     *
     * ---------------------------------------------
     **/

    /// Emit an error message based on the input error
    pub fn emit_error(&self, err: DragonDisplayError, fatal: bool) {
        let msg = Report::from_error(err).to_string();
        self.emit_by_name::<()>("error", &[&msg, &fatal]);
    }

    /// Signal emitted when the url is send
    pub fn connect_url<F: Fn(&Self, String) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "url",
            true,
            glib::closure_local!(|window, url| {
                f(window, url);
            }),
        )
    }

    /// Signal emitted when the accesstoken is send
    pub fn connect_accesstoken<F: Fn(&Self, String, String) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            "accesstoken",
            true,
            glib::closure_local!(|window, accesstoken, refreshtoken| {
                f(window, accesstoken, refreshtoken);
            }),
        )
    }

    /// Signal emitted when access token cannot be refreshed an the user needs to reconnect to
    /// googledrive
    pub fn connect_reconnect<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "reconnect",
            true,
            glib::closure_local!(|window| {
                f(window);
            }),
        )
    }

    /// Signal emitted when the api call to the total folders fails
    pub fn connect_refresh_total<F: Fn(&Self) + 'static>(&self, f: F) -> glib::SignalHandlerId {
        self.connect_closure(
            "refresh-total",
            true,
            glib::closure_local!(|window| {
                f(window);
            }),
        )
    }

    /// Signal emitted at the end of the total_folders function, indicating the total amount of
    /// folders in the target google drive
    pub fn connect_total_folders<F: Fn(&Self, u32) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            "total-folders",
            true,
            glib::closure_local!(|window, amount| {
                f(window, amount);
            }),
        )
    }

    /// Signal emitted when an error occurs
    pub fn connect_error<F: Fn(&Self, String, bool) + 'static>(
        &self,
        f: F,
    ) -> glib::SignalHandlerId {
        self.connect_closure(
            "error",
            true,
            glib::closure_local!(|window, msg, fatal| {
                f(window, msg, fatal);
            }),
        )
    }
}
