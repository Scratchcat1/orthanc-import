use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use crossbeam::channel::{Receiver, Sender};
use crate::file_upload_history::FileUploadHistory;
use crate::orthanc_types::{OrthancErrorResponse, OrthancUploadResponse};

#[derive(Debug)]
pub struct UploadResult {
    pub path: PathBuf,
    pub response: Result<OrthancUploadResponse, OrthancErrorResponse>,
}

pub fn upload_from_channel(
    files_rx: Receiver<PathBuf>,
    responses_tx: Sender<UploadResult>,
    file_upload_history: Arc<dyn FileUploadHistory + Send + Sync>,
    url: String,
    username: Option<String>,
    password: Option<String>,
) {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .expect("Failed to create HTTP Client");

    files_rx.iter().for_each(|path| {
        if file_upload_history.already_uploaded(&path) {
            println!("{} skipped",  path.display());
        } else {
            let file = File::open(&path).unwrap();
            let mut request = client.post(format!("{}/instances", url)).body(file);
            if username.is_some() && password.is_some() {
                request = request.basic_auth(username.as_ref().unwrap(), password.as_ref());
            }
            let response = match request.send() {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Failed request {}", e);
                    return ();
                }
            };

            let status = response.status();
            let text = response.text().unwrap();
            // println!("{:?}", response.text());
            // println!("HI");
            let parsed_response = if status.is_success() {
                match serde_json::from_str(&text) {
                    Ok(x) => Ok(x),
                    Err(_) => Err(OrthancErrorResponse::failed_to_parse()),
                }
            } else {
                match serde_json::from_str(&text) {
                    Ok(x) => Err(x),
                    Err(_) => Err(OrthancErrorResponse::failed_to_parse()),
                }
            };

            responses_tx
                .send(UploadResult {
                    path,
                    response: parsed_response,
                })
                .expect("channel will be there waiting for the pool");
        }
    })
}