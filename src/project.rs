use std::path::PathBuf;

pub struct Settings {
    pub source_directories: Vec<PathBuf>,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            source_directories: vec![],
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}
