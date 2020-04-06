use std::io;
use std::str;
use json;
use uuid;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Bincode(bincode::Error),
    LMDB(lmdb::Error),
    Io(io::Error),
    Json(json::Error),
    Yaml(serde_yaml::Error),
    Utf8(str::Utf8Error),
    UUID(uuid::Error),
    MalformedUUID,
    QueryType,
    QueryIterating,
    QueryUnbalanced,
    QueryUnexpectedEOS,
    QueryBadInt(std::num::ParseIntError),
    BadMetakey,
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

impl From<serde_yaml::Error> for Error {
    fn from(e: serde_yaml::Error) -> Self {
        Error::Yaml(e)
    }
}

impl From<uuid::Error> for Error {
    fn from(e: uuid::Error) -> Self {
        Error::UUID(e)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(e: std::str::Utf8Error) -> Self {
        Error::Utf8(e)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(e: std::string::FromUtf8Error) -> Self {
        Error::Utf8(e.utf8_error())
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Self {
        Error::QueryBadInt(e)
    }
}
