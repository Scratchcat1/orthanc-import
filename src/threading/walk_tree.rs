use std::path::PathBuf;

use crossbeam::channel::Sender;
use walkdir::WalkDir;

pub fn walk_dir(path: PathBuf, walk_tx: Sender<PathBuf>) {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|d| d.path().is_file())
        .for_each(|x| {
            walk_tx
                .send(x.path().to_owned())
                .expect("channel will be there waiting for the pool");
        })
}
