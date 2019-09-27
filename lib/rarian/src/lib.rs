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

mod storage;
mod error;
mod database;
mod decoders;
mod git;

pub use lmdb::{
    EnvironmentFlags,
    DatabaseFlags,
    Iter,
};

pub use error::{Result, Error};

pub use database::{
    DBManager,
    Key,
    Metadatabase,
    Stringindexdb,
    Occurance,
    find,
};
pub use storage::{
    Metadata,
    MetadataOwned,
    Metakey,
};

pub use git::{
    init,
};
