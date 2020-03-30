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

use lmdb::Transaction;

use serde::{Serialize, Deserialize};

pub struct RMDSE {
    pub dbm: DBManager,
    entry: EntryDB,
}

impl RMDSE {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<RMDSE> {
        let mut dbmb = DBManager::builder();
        dbmb.set_flags(lmdb::EnvironmentFlags::MAP_ASYNC | lmdb::EnvironmentFlags::WRITE_MAP);
        dbmb.set_max_dbs(126);
        dbmb.set_map_size(10485760);
        let dbm = DBManager::from_builder(path.as_ref(), dbmb)?;
        let entry = EntryDB::open(&dbm)?;

        Ok( Self { dbm, entry } )
    }

    pub fn query(&self) -> Result<Query> {
        let txn = self.dbm.read()?;
        Ok(Query::new(txn, self.entry))
    }

    pub fn list(&self) -> Result<()> {
        let txn = self.dbm.read()?;
        self.entry.list(&txn)
    }
}
