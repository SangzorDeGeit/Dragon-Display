use snafu::prelude::*;

#[derive(Debug, Snafu)]
pub enum DragonDisplayError {
    #[snafu(display("{}", msg), visibility(pub))]
    ConnectionRefused {
        source: std::boxed::Box<dyn serde::ser::StdError + std::marker::Send + Sync>,
        msg: String,
    },
    #[snafu(display("Failed to send message to server"), visibility(pub))]
    SendMessageError {
        source: std::sync::mpsc::SendError<()>,
    },
    #[snafu(
        display("Failed to send message from backend to manager"),
        visibility(pub)
    )]
    SendBackendError {
        source: async_channel::SendError<()>,
    },
    #[snafu(
        display(
            "There was a problem with the client secret. Follow this <a href=\"https://github.com/SangzorDeGeit/Dragon-Display/blob/main/README.md#using-google-drive\">link</a> for more info, or read the instructions on the readme for 'Using Google Drive'"
        ),
        visibility(pub)
    )]
    ClientSecretError { source: std::io::Error },
    #[snafu(display("{}", msg), visibility(pub))]
    IOError { source: std::io::Error, msg: String },
    #[snafu(display("{}", msg), visibility(pub))]
    RecvError {
        source: std::sync::mpsc::RecvError,
        msg: String,
    },
    #[snafu(display("{}", msg), visibility(pub))]
    SerializeError {
        source: toml::de::Error,
        msg: String,
    },
    #[snafu(display("{}", msg), visibility(pub))]
    ClientError {
        source: google_drive::ClientError,
        msg: String,
    },
    #[snafu(display("The inputted name is invalid: {}", msg), visibility(pub))]
    InvalidName { msg: String },
    #[snafu(display("Cannot use path: {}", msg), visibility(pub))]
    InvalidPath { msg: String },
    #[snafu(display("Address in use"), visibility(pub))]
    AddressInUse,
    #[snafu(display("{}", msg), visibility(pub))]
    InvalidData { msg: String },
    #[snafu(display("{}", msg), visibility(pub))]
    GlibError {
        source: gtk::glib::Error,
        msg: String,
    },
    #[snafu(display("{}", msg), visibility(pub))]
    DecodeError {
        source: base64::DecodeError,
        msg: String,
    },
    #[snafu(display("{}", msg), visibility(pub))]
    Other { msg: String },
}
