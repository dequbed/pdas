use crate::error::Error;
use crate::storage::Metadata;
use serde::{Serialize, Deserialize};
use libc::size_t;
use lmdb::{
    Environment,
    EnvironmentBuilder,
    Database,
    DatabaseFlags,
    Transaction,
    RoTransaction,
    RwTransaction,
    WriteFlags,
};

use std::path::Path;

type Result<T> = std::result::Result<T, Error>;

pub struct Manager {
    env: Environment
}

impl Manager {
    pub fn builder() -> EnvironmentBuilder {
        Environment::new()
    }

    pub fn from_builder(path: &Path, env: EnvironmentBuilder) -> Result<Self> {
        Ok(Self {
            env: env.open(path).map_err(Error::LMDB)?
        })
    }

    pub fn open_named(&self, name: &str) -> Result<lmdb::Database> {
        self.env.open_db(Some(name)).map_err(Error::LMDB)
    }

    pub fn create_named(&self, name: &str) -> Result<lmdb::Database> {
        self.env.create_db(Some(name), DatabaseFlags::empty()).map_err(Error::LMDB)
    }

    pub fn read(&self) -> Result<RoTransaction> {
        self.env.begin_ro_txn().map_err(Error::LMDB)
    }

    pub fn write(&self) -> Result<RwTransaction> {
        self.env.begin_rw_txn().map_err(Error::LMDB)
    }
}

/// Keytype for the Metadatabase
#[derive(Copy, Clone, Debug)]
pub struct SHA256E([u8; 32]);
impl AsRef<[u8]> for SHA256E {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// The Key used to reference a Metadata object
type Key = SHA256E;

/// The main metadata storage db
///
/// Metadata is indexed by the Key of the file it originates from
#[derive(Copy, Clone)]
pub struct Metadatabase {
    db: Database,
}
impl Metadatabase {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    fn get_bytes<'txn, T: Transaction, K: AsRef<[u8]>>(self, txn: &'txn T, key: &K) -> Result<&'txn [u8]> {
        txn.get(self.db, key).map_err(Error::LMDB)
    }

    fn reserve_bytes<'txn, K: AsRef<[u8]>>(self, txn: &'txn mut RwTransaction, key: &K, len: usize, flags: WriteFlags) -> Result<&'txn mut [u8]> {
        txn.reserve(self.db, key, len as size_t, flags).map_err(Error::LMDB)
    }

    pub fn get<'txn, T: Transaction>(self, txn: &'txn T, key: &Key) -> Result<Metadata<'txn>> {
        self.get_bytes(txn, key).and_then(Metadata::decode)
    }

    pub fn put<'txn>(self, txn: &'txn mut RwTransaction, key: &Key, m: Metadata) -> Result<()> {
        let len = m.encoded_size()? as usize;
        let buf = self.reserve_bytes(txn, key, len, WriteFlags::empty())?;
        m.encode_into(buf)
    }
}

/// An occurance of a term in a document's field.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Occurance<'txn> {
    /// The key of the document in which the term occurs
    pub key: &'txn Key,
    /// The word position where the term occurs in the document. May be multiple if the Term occurs
    /// several times.
    pub occurance: &'txn [u32],
}

#[derive(Copy, Clone)]
pub struct Stringindexdb {
    db: Database,
}
impl Stringindexdb {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    fn get_bytes<'txn, T: Transaction, K: AsRef<[u8]>>(self, txn: &'txn T, key: &K) -> Result<&'txn [u8]> {
        txn.get(self.db, key).map_err(Error::LMDB)
    }

    fn reserve_bytes<'txn, K: AsRef<[u8]>>(self, txn: &'txn mut RwTransaction, key: &K, len: usize, flags: WriteFlags) -> Result<&'txn mut [u8]> {
        txn.reserve(self.db, key, len as size_t, flags).map_err(Error::LMDB)
    }

    pub fn get<'txn, T: Transaction>(self, txn: &'txn T, key: &str) -> Result<Occurance<'txn>> {
        self.get_bytes(txn, key).and_then(|b| std::str::from_utf8(b).map_err(Error::Utf8))
    }

    pub fn put<'txn>(self, txn: &'txn mut RwTransaction, key: &str, value: Occurance<'txn>) -> Result<()> {
        let len = bincode::serialized_size(value)? as usize;
        let buf = self.reserve_bytes(txn, key, len, WriteFlags::empty())?;
        bincode::serialize_into(buf, value)
    }
}
