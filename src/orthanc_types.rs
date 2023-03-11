use serde::Deserialize;
use std::fmt::{self, Debug};

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum OrthancUploadStatus {
    AlreadyStored,
    Success,
}

impl fmt::Display for OrthancUploadStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let status = match self {
            Self::AlreadyStored => "Already Stored",
            Self::Success => "Success",
        };
        write!(f, "{}", status)
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum OrthancUploadResponse {
    OrthancDicomUploadResponse(OrthancDicomUploadResponse),
    OrthancFolderUploadResponse(Vec<OrthancDicomUploadResponse>),
}

impl OrthancUploadResponse {
    pub fn success_message(&self) -> String {
        match self {
            Self::OrthancDicomUploadResponse(inner) => match inner.status {
                OrthancUploadStatus::AlreadyStored => "Already Stored".to_string(),
                OrthancUploadStatus::Success => "Success".to_string(),
            },
            Self::OrthancFolderUploadResponse(inner) => {
                let new = inner
                    .iter()
                    .filter(|x| x.status == OrthancUploadStatus::Success)
                    .count();
                let total = inner.len();
                format!("({}/{}) (New/Total)", new, total)
            }
        }
    }
}

impl fmt::Display for OrthancUploadResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::OrthancDicomUploadResponse(inner) => std::fmt::Display::fmt(&inner, f),
            Self::OrthancFolderUploadResponse(inner) => {
                let strings: Vec<String> = inner.iter().map(|x| format!("{}", x)).collect();
                write!(f, "{}", strings.join("\n"))
            }
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct OrthancDicomUploadResponse {
    #[serde(alias = "ID")]
    pub id: String,
    #[serde(alias = "ParentPatient")]
    pub parent_patient: String,
    #[serde(alias = "ParentSeries")]
    pub parent_series: String,
    #[serde(alias = "ParentStudy")]
    pub parent_study: String,
    #[serde(alias = "Path")]
    pub path: String,
    #[serde(alias = "Status")]
    pub status: OrthancUploadStatus,
}

impl fmt::Display for OrthancDicomUploadResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "OrthancUploadResponse:
    ID: {}
    Orthanc Patient ID: {}
    Orthanc Series ID: {}
    Orthanc Study ID: {}
    Stored Path: {}
    Status: {}",
            self.id,
            self.parent_patient,
            self.parent_series,
            self.parent_study,
            self.path,
            self.status
        )
    }
}

#[derive(Deserialize, Debug)]
pub struct OrthancErrorResponse {
    #[serde(alias = "Details")]
    pub details: String,
    #[serde(alias = "HttpError")]
    pub http_error: String,
    #[serde(alias = "HttpStatus")]
    pub http_status: u32,
    #[serde(alias = "Message")]
    pub message: String,
    #[serde(alias = "Method")]
    pub method: String,
    #[serde(alias = "OrthancError")]
    pub orthanc_error: String,
    #[serde(alias = "OrthancStatus")]
    pub orthanc_status: i64,
    #[serde(alias = "Uri")]
    pub uri: String,
}

impl fmt::Display for OrthancErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "OrthancErrorResponse:
    Details: {}
    Http Error: {}
    Http Status: {}
    Message: {}
    Method: {}
    Orthanc Error: {}
    Orthanc Status: {}
    Uri: {}",
            self.details,
            self.http_error,
            self.http_status,
            self.message,
            self.method,
            self.orthanc_error,
            self.orthanc_status,
            self.uri,
        )
    }
}
