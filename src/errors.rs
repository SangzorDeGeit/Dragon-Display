use serde::ser::StdError;
use snafu::prelude::*;
use thiserror::Error;

#[derive(Debug, Snafu)]
pub enum DragonDisplayError {
    #[snafu(display("{}", msg), visibility(pub))]
    ConnectionRefused {
        source: std::boxed::Box<dyn StdError + std::marker::Send + Sync>,
        msg: String,
    },
    #[snafu(display("Failed to send message to server"), visibility(pub))]
    SendMessageError {
        source: std::sync::mpsc::SendError<()>,
    },
    #[snafu(
        display("There was a problem with the client secret, follow the instructions on the github page for more info"),
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
    Other { msg: String },
}

#[derive(Error, Debug)]
pub enum GoogleDriveError {
    #[error("The following files could not be downloaded {}", files.join(", "))]
    DownloadFailed { files: Vec<String> },
    #[error("Failed to connect to google drive")]
    ConnectionFailed,
    #[error("Google drive refused the connection")]
    ConnectionRefused,
    #[error("client_secret.json not found. In order to use google drive please follow the steps in the readme on github")]
    ClientSecretNotFound,
    #[error("Some uknown error happened while trying to read the client secret, follow the steps on the readme on github to configure google drive")]
    ClientSecretError,
    #[error("Did not have permission to read the client secret")]
    PermissionDenied,
    #[error("Could not remove file: {}", file)]
    RemoveFile { file: String },
    #[error("Internal error: called drive function for a non-google drive campaign")]
    InternalError,
}
