use std::{path::Path, time::Duration};

use reqwest::blocking::Client;

use crate::orthanc::{
    types::{OrthancErrorResponse, OrthancUploadResponse},
    upload::upload_dicom,
};

#[derive(Clone)]
pub struct OrthancConnector {
    client: Client,
    username: Option<String>,
    password: Option<String>,
    url: String,
}

impl OrthancConnector {
    pub fn new(username: Option<String>, password: Option<String>, url: String) -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP Client");

        Self {
            client,
            username,
            password,
            url,
        }
    }

    pub fn upload_file(&self, path: &Path) -> Result<OrthancUploadResponse, OrthancErrorResponse> {
        upload_dicom(
            &self.client,
            &self.url,
            self.username.as_deref(),
            self.password.as_deref(),
            path,
        )
    }
}
