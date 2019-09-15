use std::io;
use json;

#[derive(Debug)]
pub enum Error {
    Directory,
    Bincode(bincode::Error),
    LMDB(lmdb::Error),
    Io(io::Error),
    Json(json::Error),
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
