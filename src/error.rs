use std::fmt;
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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::SystemNotSupport => write!(f, "system not support"),
            Error::FileNotExists(path) => write!(f, "file not exists: {:?}", path),
            Error::SearchFailed => write!(f, "search failed"),
            Error::IoError(e) => write!(f, "io error: {}", e),
            Error::ZipError((e, path)) => write!(f, "zip error: {}, path: {:?}", e, path),
        }
    }
}
