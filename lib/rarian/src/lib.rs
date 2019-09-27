#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

extern crate bincode;
extern crate chrono;
extern crate git2;
extern crate libc;
extern crate lmdb;
extern crate rust_stemmers;
extern crate serde;
extern crate tree_magic;

#[cfg(epub)]
extern crate epub;
#[cfg(flac)]
extern crate metaflac;
#[cfg(id3)]
extern crate id3;

#[cfg(test)]
#[macro_use]
extern crate maplit;

mod storage;
mod error;
mod database;
mod decoders;

pub use lmdb::{
    EnvironmentFlags,
};

pub use database::Manager;
pub use error::{Result, Error};
