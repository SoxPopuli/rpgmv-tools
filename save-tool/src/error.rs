use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Global(String),
    Io(String),
    Lib(lib::error::Error),
    Image(String),
    MissingDirectory(PathBuf),
}
impl Error {
    pub fn global_error(s: impl Into<String>) -> Self {
        Self::Global(s.into())
    }

    pub fn io_error(s: impl Into<String>) -> Self {
        Self::Io(s.into())
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}
impl From<lib::error::Error> for Error {
    fn from(value: lib::error::Error) -> Self {
        Self::Lib(value)
    }
}
impl From<image::ImageError> for Error {
    fn from(value: image::ImageError) -> Self {
        Self::Image(value.to_string())
    }
}
impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}
