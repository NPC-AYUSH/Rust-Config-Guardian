use std::path::Path;

pub fn is_valid_directory(path: &str) -> bool {
    Path::new(path).is_dir()
}
