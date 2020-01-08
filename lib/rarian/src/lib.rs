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

use std::path::Path;
use std::collections::HashMap;

use error::Result;
use index::Indexer;
use db::{dbm::DBManager, EntryDB};
use db::entry::{EntryT, UUID};

use serde::{Serialize, Deserialize};

pub struct RMDSE {
    dbm: DBManager,
    index: Indexer,
}

impl RMDSE {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<RMDSE> {
        let dbmb = DBManager::builder();
        let dbm = DBManager::from_builder(path.as_ref(), dbmb)?;
        let entry = dbm.open_named("entry")?;
        let index = Indexer::new(EntryDB::new(entry), HashMap::new());

        Ok( Self { dbm, index } )
    }

    pub fn indexer(&mut self) -> &mut Indexer {
        &mut self.index
    }

    pub fn dbm(&mut self) -> &mut DBManager {
        &mut self.dbm
    }
}
