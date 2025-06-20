use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::io::{BufRead, BufReader, BufWriter};
use std::path::PathBuf;
use std::sync::RwLock;

pub trait FileUploadHistory {
    fn already_uploaded(&self, path: &PathBuf) -> bool;
    fn on_success(&self, path: &PathBuf);
}

pub struct DisabledFileUploadHistory;
impl FileUploadHistory for DisabledFileUploadHistory {
    fn already_uploaded(&self, _path: &PathBuf) -> bool {
        false
    }

    fn on_success(&self, _path: &PathBuf) {}
}

pub struct TextFileUploadHistory {
    history_path: PathBuf,
    paths: RwLock<HashSet<PathBuf>>,
}

impl TextFileUploadHistory {
    pub fn from_file(path_buf: &PathBuf) -> TextFileUploadHistory {
        let paths = match File::open(path_buf) {
            Ok(f) => read_path_set_from_file(f).unwrap(),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => HashSet::new(),
            Err(_) => panic!("oh no"),
        };
        TextFileUploadHistory {
            history_path: path_buf.clone(),
            paths: RwLock::new(paths),
        }
    }

    fn save_to_file(&self, new_path: &PathBuf) -> std::io::Result<()> {
        let file = File::options().append(true).open(&self.history_path)?;
        let mut writer = BufWriter::new(file);
        writeln!(writer, "{}", new_path.display())?;
        Ok(())
    }
}

impl FileUploadHistory for TextFileUploadHistory {
    fn already_uploaded(&self, path: &PathBuf) -> bool {
        self.paths.read().unwrap().contains(path)
    }

    fn on_success(&self, path: &PathBuf) {
        let mut paths = self.paths.write().unwrap();
        paths.insert(path.clone());
        self.save_to_file(path).unwrap()
    }
}

fn read_path_set_from_file(file: File) -> std::io::Result<HashSet<PathBuf>> {
    let reader = BufReader::new(file);

    let mut path_set = HashSet::new();

    for line in reader.lines() {
        let line = line?; // Handle I/O errors
        let trimmed = line.trim();

        if !trimmed.is_empty() {
            path_set.insert(PathBuf::from(trimmed));
        }
    }

    Ok(path_set)
}
