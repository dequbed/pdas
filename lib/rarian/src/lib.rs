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
// Creation, updating & managing of indices
pub mod index;
// Querying indices and entries
pub mod query;

pub mod schema;

use std::path::Path;
use std::collections::HashMap;

use error::Result;
use index::Indexer;
use db::{dbm::DBManager, EntryDB};
use db::entry::{EntryT, UUID};
use query::Query;

use serde::{Serialize, Deserialize};

pub struct RMDSE {
    dbm: DBManager,
    entry: EntryDB,
}

impl RMDSE {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<RMDSE> {
        let mut dbmb = DBManager::builder();
        dbmb.set_flags(lmdb::EnvironmentFlags::MAP_ASYNC | lmdb::EnvironmentFlags::WRITE_MAP);
        dbmb.set_map_size(10485760);
        dbmb.set_max_dbs(4);
        let dbm = DBManager::from_builder(path.as_ref(), dbmb)?;
        let entry = EntryDB::open(&dbm)?;

        Ok( Self { dbm, entry } )
    }

    pub fn indexer(&self) -> Result<Indexer> {
        let txn = self.dbm.write()?;
        Ok(Indexer::new(txn, self.entry, HashMap::new()))
    }

    pub fn query(&self) -> Result<Query> {
        let txn = self.dbm.read()?;
        Ok(Query::new(txn, self.entry))
    }

    pub fn export<P: AsRef<Path>>(&self, dir: P) -> Result<()> {
        let txn = self.dbm.read()?;
        self.entry.export(dir.as_ref(), &txn)
    }
}
