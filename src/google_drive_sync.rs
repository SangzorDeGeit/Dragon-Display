use google_drive::Client;

pub async fn initialize() {
    println!("Starting...");
    let mut google_drive_client = Client::new(
        "1043613452788-2rq3ksqhaivjtt5hjjp5o49a0n87nbh2.apps.googleusercontent.com".to_string(),
        "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".to_string(),
        "https://www.googleapis.com/drive/v3/files".to_string(),
        "",
        "",
    );
        
    println!("made client");
    let user_consent_url = google_drive_client.user_consent_url(&["https://www.googleapis.com/auth/drive.readonly".to_string()]);
    println!("The consent url: {}", user_consent_url);

    let code = "4";
    let state = "55b283b7-b22a-4da6-b3ce-58f10647adbd";
    let mut access_token = google_drive_client.get_access_token(code, state).await.unwrap();
    

}

pub async fn something() {
    println!("Hello, this is a test!");
}

