use std::path::PathBuf;

use super::util::FileUploadHistory;

#[derive(Clone, Copy, Debug, Default)]
pub struct DisabledFileUploadHistory;

impl FileUploadHistory for DisabledFileUploadHistory {
    type ID = u64;
    fn check(&self, _path: &PathBuf) -> Option<Self::ID> {
        return Some(0);
    }

    fn on_upload(&self, _id: &Self::ID) {}
}
