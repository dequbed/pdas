use std::collections::HashSet;

use libc::size_t;
use lmdb::{
    Database,
    Transaction,
    RwTransaction,
    RoTransaction,
    WriteFlags,
    Iter,
    Cursor,
};
use serde::{
    Deserialize,
    Serialize,
};

use crate::error::{Result, Error};
use crate::db::entry::UUID;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Matches(HashSet<UUID>);

impl Matches {
    pub fn new(set: HashSet<UUID>) -> Self {
        Self ( set )
    }

    pub fn into_set(self) -> HashSet<UUID> {
        self.0
    }

    pub fn encoded_size(&self) -> Result<u64> {
        bincode::serialized_size(&self.0).map_err(Error::Bincode)
    }

    pub fn encode_into(&self, bytes: &mut [u8]) -> Result<()> {
        bincode::serialize_into(bytes, &self).map_err(Error::Bincode)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes).map_err(Error::Bincode)
    }

    pub fn combine(&mut self, other: &Matches) {
        let union = self.0.union(&other.0);
        self.0 = union.map(|x| *x).collect();
    }
}

#[derive(Copy, Clone)]
pub struct TitleDB {
    db: Database,
}

impl TitleDB {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    fn get_bytes<'txn, T: Transaction, K: AsRef<[u8]>>(self, txn: &'txn T, key: &K) -> Result<&'txn [u8]> {
        txn.get(self.db, key).map_err(Error::LMDB)
    }

    fn reserve_bytes<'txn, K: AsRef<[u8]>>(self, txn: &'txn mut RwTransaction, key: &K, len: usize, flags: WriteFlags) -> Result<&'txn mut [u8]> {
        txn.reserve(self.db, key, len as size_t, flags).map_err(Error::LMDB)
    }

    pub fn get<'txn, T: Transaction>(self, txn: &'txn T, key: &str) -> Result<Matches> {
        self.get_bytes(txn, &key).and_then(Matches::decode)
    }

    pub fn put<'txn>(self, txn: &'txn mut RwTransaction, key: &str, m: Matches) -> Result<()>
    {
        let len = m.encoded_size()? as usize;
        let buf = self.reserve_bytes(txn, &key, len, WriteFlags::empty())?;
        m.encode_into(buf)
    }

    pub fn iter_start<'txn, T: Transaction>(self, txn: &'txn T) -> Result<Iter<'txn>> {
        let mut cursor = txn.open_ro_cursor(self.db)?;
        Ok(cursor.iter_start())
    }

    pub fn insert_match<'txn>(&mut self, txn: &'txn mut RwTransaction, key: &str, uuid: UUID) -> Result<bool> {
        match self.get(txn, key) {
            Ok(matches) => {
                let mut matches = matches.into_set();
                let r = matches.insert(uuid);
                self.put(txn, key, Matches::new(matches))?;

                Ok(r)
            }
            Err(Error::LMDB(lmdb::Error::NotFound)) => {
                let mut matches = HashSet::new();
                let r = matches.insert(uuid);
                self.put(txn, key, Matches::new(matches))?;

                Ok(r)
            }
            Err(e) => return Err(e),
        }
    }

    pub fn insert_matches<'txn>(&mut self, txn: &'txn mut RwTransaction, key: &str, other: &Matches) -> Result<()> {
        match self.get(txn, key) {
            Ok(mut m) => {
                m.combine(other);
                self.put(txn, key, m)?;

                Ok(())
            }
            Err(Error::LMDB(lmdb::Error::NotFound)) => {
                self.put(txn, key, other.clone())?;

                Ok(())
            }
            Err(e) => return Err(e),
        }
    }
}
