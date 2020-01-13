use crate::error::*;

use crate::db::{
    EntryDB,
    entry::Entry,
    entry::UUID,
};

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
