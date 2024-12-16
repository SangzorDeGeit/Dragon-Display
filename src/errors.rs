use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("The config file got corrupted. Please remove the .config.toml file (a hidden file in this directory) and restart the application")]
    InvalidConfig,
    #[error("Too many campaigns found in the config file. Please remove the .config.toml file (a hidden file in this directory) and restart the application")]
    TooManyCampaigns,
    #[error("Found a duplicate in the config file. Please remove the .config.toml file (a hidden file in this directory) and restart the application")]
    DuplicateCampaign,
    #[error("Could not get permission to access the config file")]
    PermissionDenied,
    #[error("Could not find the campaign to be removed")]
    CampaignNotFound,
    #[error("Could not remove campaign folder: found non-media files")]
    CouldNotRemove,
    #[error("Could not create the folder to put the images in")]
    FolderCreationError,
    #[error("An internal error occurred")]
    Other,
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
