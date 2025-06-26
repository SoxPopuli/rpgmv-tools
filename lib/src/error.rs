use serde_json::Error as JsonError;

#[derive(Debug, PartialEq, Eq)]
pub enum LzErrorKind {
    Compression,
    Decompression,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Lz(LzErrorKind),
    Json(String),
    Key(String),
    Io(std::io::ErrorKind),
}
impl From<JsonError> for Error {
    fn from(value: JsonError) -> Self {
        let s = value.to_string();
        Self::Json(s)
    }
}
impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.kind())
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}
