use std::io;
use toml;
use rarian;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Toml(toml::de::Error),
    Rarian(rarian::Error),
    Io(io::Error),
    Directory,
}

impl From<rarian::Error> for Error {
    fn from(e: rarian::Error) -> Self {
        use rarian::Error::*;
        match e {
            Io(e) => Error::Io(e),
            _ => Error::Rarian(e),
        }
    }
}

impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self {
        Error::Toml(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}
