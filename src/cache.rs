use std::io::Write;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter};
use std::path::PathBuf;

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

#[derive(Clone)]
pub struct Cache {
    pub paths: HashSet<PathBuf>,
}

impl Cache {
    pub fn from_file(path_buf: &PathBuf) -> Cache {
        let paths = match File::open(path_buf) {
            Ok(f) => read_path_set_from_file(f).unwrap(),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => HashSet::new(),
            Err(_) => panic!("oh no"),
        };
        Cache { paths }
    }

    pub fn save_to_file(&self, path_buf: PathBuf) -> std::io::Result<()> {
        let file = File::create(path_buf)?;
        let mut writer =  BufWriter::new(file);
        let mut sorted_paths: Vec<_> = self.paths.iter().collect();
        sorted_paths.sort();

        for path in sorted_paths {
            writeln!(writer, "{}", path.display())?;
        }
        Ok(())
    }
}