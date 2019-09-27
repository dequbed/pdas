use std::io;
use std::str;
use json;
use toml;
use crate::decoders;

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Directory,
    Bincode(bincode::Error),
    LMDB(lmdb::Error),
    Io(io::Error),
    Json(json::Error),
    Toml(toml::de::Error),
    Utf8(str::Utf8Error),
    Decode(decoders::DecodeError),
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
impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self {
        Error::Toml(e)
    }
}
impl From<decoders::DecodeError> for Error {
    fn from(e: decoders::DecodeError) -> Self {
        Error::Decode(e)
    }
}
