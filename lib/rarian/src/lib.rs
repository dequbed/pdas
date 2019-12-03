#![allow(unused_imports)]
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

mod storage;
mod error;
mod database;
mod decoders;
mod decoder;
mod git;
mod archive;

pub mod db;
pub mod index;

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
