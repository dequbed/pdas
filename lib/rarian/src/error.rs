use std::io;
use std::str;
use json;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Bincode(bincode::Error),
    LMDB(lmdb::Error),
    Io(io::Error),
    Json(json::Error),
    Utf8(str::Utf8Error),
}

impl From<bincode::Error> for Error {
    fn from(e: bincode::Error) -> Self {
        Error::Bincode(e)
    }
}

impl From<lmdb::Error> for Error {
    fn from(e: lmdb::Error) -> Self {
        Error::LMDB(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<json::Error> for Error {
    fn from(e: json::Error) -> Self {
        Error::Json(e)
    }
}
