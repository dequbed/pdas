#![allow(unused_imports)]
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate futures;

mod error;

// Storage of entries & indices
pub mod db;
// Querying indices and entries
pub mod query;

pub mod schema;

mod uuid;

use std::path::Path;
use std::collections::HashMap;
use std::mem;

use error::Result;
use db::{dbm::DBManager, EntryDB};
use db::entry::EntryT;
use query::Query;

pub use lmdb::{
    Transaction,
    RwTransaction,
};

use serde::{Serialize, Deserialize};
