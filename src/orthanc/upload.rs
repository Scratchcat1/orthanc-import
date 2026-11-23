use crate::orthanc::types::{OrthancErrorResponse, OrthancUploadResponse};
use reqwest::blocking::Client;
use std::fs::File;
use std::path::PathBuf;

pub fn upload_dicom(
    client: &Client,
    url: &str,
    username: Option<&str>,
    password: Option<&str>,
    path: &PathBuf,
) -> Result<OrthancUploadResponse, OrthancErrorResponse> {
    let file = File::open(&path).unwrap();
    let mut request = client.post(format!("{}/instances", url)).body(file);
    if username.is_some() && password.is_some() {
        request = request.basic_auth(username.unwrap(), password);
    }
    let response = match request.send() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed request {}", e);
            todo!("urgh oh");
        }
    };

    let status = response.status();
    let text = response.text().unwrap();

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

    parsed_response
}
