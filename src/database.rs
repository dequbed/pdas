use crate::error::{Result, Error};
use crate::storage::{Metadata, MetadataS};
use serde::{Serialize, Deserialize};
use libc::size_t;

pub use lmdb::{
    Environment,
    EnvironmentBuilder,
    Database,
    DatabaseFlags,
    Transaction,
    RoTransaction,
    RwTransaction,
    WriteFlags,
    RoCursor,
    Cursor,
    Iter,
    IterDup,
};

use std::path::Path;

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
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SHA256E([u8; 32]);
impl AsRef<[u8]> for SHA256E {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
impl SHA256E {
    pub fn try_parse(s: &str) -> Option<Self> {
        let mut si = s.split("--");
        let [a,b]: [&str; 2] = [si.next().unwrap(), si.next().unwrap()];
        let mut info = a.split('-');
        if let Some(m) = info.next() {
            if m == "SHA256E" {
                if let Some(k) = b.split('.').next() {
                    let mut inner = [0u8;32];

                    for (idx, pair) in k.as_bytes().chunks(2).enumerate() {
                        inner[idx] = val(pair[0]) << 4 | val(pair[1])
                    }

                    return Some(Self(inner));
                }
            }
        }

        None
    }
}

fn val(c: u8) -> u8 {
    match c {
        b'A'...b'F' => c - b'A' + 10,
        b'a'...b'f' => c - b'a' + 10,
        b'0'...b'9' => c - b'0',
        _ => 0
    }
}


/// The Key used to reference a Metadata object
pub type Key = SHA256E;

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

    pub fn put<'txn, S, B>(self, txn: &'txn mut RwTransaction, key: &Key, m: MetadataS<S,B>) -> Result<()> 
        where S: AsRef<str> + Serialize + Deserialize<'txn>,
              B: AsRef<[u8]> + Serialize + Deserialize<'txn>
    {
        let len = m.encoded_size()? as usize;
        let buf = self.reserve_bytes(txn, key, len, WriteFlags::empty())?;
        m.encode_into(buf)
    }

    pub fn iter_start<'txn, T: Transaction>(self, txn: &'txn T) -> Result<Iter<'txn>> {
        let mut cursor = txn.open_ro_cursor(self.db)?;
        Ok(cursor.iter_start())
    }
}

/// An occurance of a term in a document's field.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Occurance {
    /// The key of the document in which the term occurs
    pub key: Key,
    /// The word position where the term occurs in the document. May be multiple if the Term occurs
    /// several times.
    pub occurance: Vec<u32>,
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

    pub fn get<'txn, T: Transaction>(self, txn: &'txn T, key: &str) -> Result<Occurance> {
        self.get_bytes(txn, &key).and_then(|buf| bincode::deserialize::<Occurance>(buf).map_err(Error::Bincode))
    }

    pub fn put<'txn>(self, txn: &'txn mut RwTransaction, key: &str, value: &Occurance) -> Result<()> {
        let len = bincode::serialized_size(value)? as usize;
        let buf = self.reserve_bytes(txn, &key, len, WriteFlags::empty())?;
        bincode::serialize_into(buf, value).map_err(Error::Bincode)
    }

    pub fn iter_start<'txn, T: Transaction>(self, txn: &'txn T) -> Result<IterDup<'txn>> {
        let mut cursor = txn.open_ro_cursor(self.db)?;
        Ok(cursor.iter_dup_start())
    }

    pub fn iter<'txn, T: Transaction>(self, txn: &'txn T, key: &str) -> Result<Iter<'txn>> {
        let mut cursor = txn.open_ro_cursor(self.db)?;
        Ok(cursor.iter_dup_of(key))
    }

    pub fn delete<'txn>(self, txn: &'txn mut RwTransaction, key: &str, value: &Occurance) -> Result<()> {
        let val = bincode::serialize(value)?;
        txn.del(self.db, &key, Some(&val)).map_err(Error::LMDB)
    }
}
