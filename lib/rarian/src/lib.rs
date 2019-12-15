#![allow(unused_imports)]
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

mod storage;
mod error;
mod database;

pub mod db;
pub mod index;
pub mod decode;

pub use lmdb::{
    EnvironmentFlags,
    DatabaseFlags,
    Iter,
};

pub use error::{Result, Error};

pub use database::{
    DBManager,
    Key,
};
pub use storage::{
    Metadata,
    MetadataOwned,
    Metakey,
};
