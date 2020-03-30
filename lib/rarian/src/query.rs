use std::ops::Bound;

use crate::error::*;

use crate::db::{
    EntryDB,
    RangeDB,
    entry::Entry,
};

use crate::uuid::UUID;

use crate::db::dbm::{
    DBManager,
    RoTransaction,
};

pub struct Querier {
    dbm: DBManager,
}

impl Querier {
    pub fn new(dbm: DBManager) -> Self {
        Self { dbm }
    }

    pub fn query<'txn>(&'txn mut self) -> Result<Query<'txn>> {
        let txn = self.dbm.read()?;

        let entrydb = EntryDB::open(&self.dbm)?;

        Ok( Query { txn, entrydb } )
    }
}

pub struct Query<'txn> {
    txn: RoTransaction<'txn>,
    entrydb: EntryDB
}

impl<'txn, 't> Query<'txn> {
    pub fn new(txn: RoTransaction<'txn>, entrydb: EntryDB) -> Query<'txn> {
        Self { txn, entrydb }
    }

    pub fn retrieve(&'t mut self, uuid: UUID) -> Result<Entry<'t>> {
        self.entrydb.get(&self.txn, &uuid)
    }
}

#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub struct RangeQuery {
    pub upper: Bound<u64>,
    pub lower: Bound<u64>,
}

impl RangeQuery {
    pub fn new(upper: Bound<u64>, lower: Bound<u64>) -> Self {
        Self { upper, lower }
    }

    pub fn run(self, db: &RangeDB) -> impl Iterator<Item = (&u64, &UUID)> {
        db.range((self.lower,self.upper))
    }
}

// Query: Takes specific index, gives a Set of valid UUIDs
//   Range
//   Text match
//
// Combiner: Take multiple sets of UUIDs, combines into one
//   AND
//   OR
//
// Transformers (get a single set)
//   Sorting: Sort Set of UUIDs
//   Filter: filter by something
