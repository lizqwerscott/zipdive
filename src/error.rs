use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Error {
    SystemNotSupport,
    FileNotExists(PathBuf),
    SearchFailed,
    IoError(String),
    ZipError((String, PathBuf)),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError(e.to_string())
    }
}
