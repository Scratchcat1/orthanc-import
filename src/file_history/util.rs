use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use data_encoding::HEXLOWER;
use sha2::Digest;

pub trait FileUploadHistory {
    type ID;
    fn check(&self, path: &Path) -> Option<Self::ID>;
    fn on_upload(&self, id: &Self::ID);
}

pub fn digest_file<D: Digest>(path: &Path) -> String {
    let mut reader = BufReader::new(File::open(path).unwrap());
    let mut hasher = D::new();
    let mut buffer = [0; 65536];

    loop {
        let count = reader.read(&mut buffer).unwrap();
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    HEXLOWER.encode(&hasher.finalize())
}
