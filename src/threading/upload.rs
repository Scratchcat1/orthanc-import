use std::path::PathBuf;

use crate::file_history::util::FileUploadHistory;
use crate::orthanc::connector::OrthancConnector;
use crate::orthanc::types::{OrthancErrorResponse, OrthancUploadResponse};
use crossbeam::channel::{Receiver, Sender};

#[derive(Debug)]
pub struct UploadResult<ID> {
    pub id: ID,
    pub path: PathBuf,
    pub response: Result<OrthancUploadResponse, OrthancErrorResponse>,
}

pub fn upload_from_channel<H: FileUploadHistory<ID = R>, R>(
    connector: &OrthancConnector,
    file_history: H,
    files_rx: Receiver<PathBuf>,
    responses_tx: Sender<UploadResult<R>>,
) {
    files_rx.iter().for_each(|path| {
        if let Some(id) = file_history.check(&path) {
            let response = connector.upload_file(&path);

            responses_tx
                .send(UploadResult {
                    id: id,
                    path: path.to_owned(),
                    response: response,
                })
                .expect("channel will be there waiting for the pool");
        } else {
            println!("{} skipped", path.display());
        }
    })
}
