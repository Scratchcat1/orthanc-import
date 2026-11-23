use std::path::Path;

use crate::file_history::{
    disabled::DisabledFileUploadHistory, sqlite::SqliteFileUploadHistory, util::FileUploadHistory,
};

pub mod disabled;
pub mod sqlite;
pub mod util;

#[derive(Clone)]
pub enum FileUploadHistoryImpl {
    Sqlite(SqliteFileUploadHistory),
    Disabled(DisabledFileUploadHistory),
}

impl FileUploadHistory for FileUploadHistoryImpl {
    type ID = u64;

    fn check(&self, path: &Path) -> Option<Self::ID> {
        match self {
            Self::Sqlite(history) => history.check(path),
            Self::Disabled(history) => history.check(path),
        }
    }

    fn on_upload(&self, id: &u64) {
        match self {
            Self::Sqlite(history) => history.on_upload(id),
            Self::Disabled(history) => history.on_upload(id),
        }
    }
}
