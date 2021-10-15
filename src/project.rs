use std::path::PathBuf;

pub struct Settings {
    pub source_directories: Vec<PathBuf>,
}

impl Settings {
    pub fn new() -> Settings {
        Settings {
            source_directories: vec![],
        }
    }
}
