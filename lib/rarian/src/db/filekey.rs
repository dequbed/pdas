use std::io::Write;

use libc::size_t;

use lmdb::{
    Database,
    Transaction,
    RwTransaction,
    WriteFlags,
};

use crate::error::{Result, Error};

use crate::db::entry::FileKey;
use crate::uuid::UUID;

#[derive(Copy, Clone)]
/// Reverse Entry index for files
///
/// Useful to look up the entry for a specified file and also to figure out if we already know a
/// file and thus don't need to index it.
pub struct FilekeyDB {
    db: Database,
}

impl FilekeyDB {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    fn get_bytes<'txn, T: Transaction, K: AsRef<[u8]>>(self, txn: &'txn T, key: &K) -> Result<&'txn [u8]> {
        txn.get(self.db, key).map_err(Error::LMDB)
    }

    fn reserve_bytes<'txn, K: AsRef<[u8]>>(self, txn: &'txn mut RwTransaction, key: &K, len: usize, flags: WriteFlags) -> Result<&'txn mut [u8]> {
        txn.reserve(self.db, key, len as size_t, flags).map_err(Error::LMDB)
    }

    pub fn put<'txn>(self, txn: &mut RwTransaction, filekey: &FileKey, uuid: &UUID) -> Result<()>
    {
        let len = uuid.encoded_size() as usize;
        let mut buf = self.reserve_bytes(txn, &filekey.as_bytes(), len, WriteFlags::empty())?;
        buf.write(&uuid.as_bytes())?;

        Ok(())
    }

    pub fn get<'txn, T: Transaction>(self, txn: &'txn T, key: &FileKey) -> Result<UUID> {
        self.get_bytes(txn, &key.as_bytes()).and_then(UUID::from_bytes)
    }

}
