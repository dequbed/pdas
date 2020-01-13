use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::iter::FromIterator;

use std::hash::{Hash, Hasher};

use uuid::Uuid;

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

use crate::db::dbm::DBManager;

use crate::db::meta::Metakey;
use crate::error::{Result, Error};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct UUID(u128);

impl UUID {
    pub fn new(uuid: Uuid) -> Self{
        Self (uuid.as_u128())
    }
    pub fn generate() -> Self {
        let u = Uuid::new_v4();
        Self::new(u)
    }
    pub fn as_uuid(self) -> Uuid {
        Uuid::from_u128(self.0)
    }
    pub fn as_bytes(self) -> [u8; 16] {
        self.0.to_le_bytes()
    }
}

type FileKey = String;
type FormatKey = u32;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A file indexed by git-annex
///
/// Important: This struct has custom `Eq` and `Hash` behaviour in that only the key will be
/// considered, format metadata is ignored.
pub struct FileT<B> {
    key: FileKey,
    format: HashMap<FormatKey, B>,
}
impl<B> PartialEq for FileT<B> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}
impl<B> Eq for FileT<B> {}
impl<B> Hash for FileT<B> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

impl<B> FileT<B> {
    pub fn new(key: FileKey, format: HashMap<FormatKey, B>) -> Self {
        Self { key, format}
    }
}

impl<B: AsRef<[u8]>> FileT<B> {
    pub fn ref_eq<A>(&self, other: &FileT<A>) -> bool
        where A: AsRef<[u8]>
    {
        self.key == other.key &&
            std::iter::Iterator::eq(
                self.format.iter().map(|(k,v)| (k, v.as_ref())), 
                other.format.iter().map(|(k,v)| (k, v.as_ref())))
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
/// A metadata entry
///
/// Each entry connects a metadata map to a set of filekeys. All filekey should identifiy the
/// logically same file although its actual bits may be different. For example the same song
/// encoded in FLAC and ogg/vorbis should be the same entry, and the same song but with
/// Vorbis comments or ID3 tags attached / not attached should be the same entry.
pub struct EntryT<B> {
    files: HashSet<FileT<B>>,
    /// Metadata is an arbitrary key-value map
    metadata: HashMap<Metakey, B>,
}
impl<'e, B> EntryT<B>
    where B: Serialize + Deserialize<'e> + AsRef<[u8]>,
{
    pub fn new(filekey: FileT<B>, metadata: HashMap<Metakey, B>) -> Self {
        let mut set = HashSet::new();
        set.insert(filekey);

        Self::newv(set, metadata)
    }

    pub fn newv(files: HashSet<FileT<B>>, metadata: HashMap<Metakey, B>) -> Self {
        Self {
            files,
            metadata,
        }
    }

    pub fn decode(bytes: &'e [u8]) -> Result<Self> {
        bincode::deserialize(bytes).map_err(Error::Bincode)
    }

    pub fn encode_into(&self, bytes: &mut [u8]) -> Result<()> {
        bincode::serialize_into(bytes, &self).map_err(Error::Bincode)
    }

    pub fn encoded_size(&self) -> Result<u64> {
        bincode::serialized_size(self).map_err(Error::Bincode)
    }

    pub fn keys(&self) -> &HashSet<FileT<B>> {
        &self.files
    }

    pub fn metadata(&self) -> &HashMap<Metakey, B> {
        &self.metadata
    }

    pub fn to_yaml(&self) -> std::result::Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }

    pub fn meta_ref_eq<A>(&self, other: &EntryT<A>) -> bool
        where A: AsRef<[u8]>
    {
        std::iter::Iterator::eq(
            self.metadata.iter().map(|(k,v)| (k, v.as_ref())), 
            other.metadata.iter().map(|(k,v)| (k, v.as_ref())))
    }
}
impl<B: Eq> PartialEq for EntryT<B> {
    fn eq(&self, other: &Self) -> bool {
        self.files == other.files && 
            std::iter::Iterator::eq(self.metadata.iter(), other.metadata.iter())
    }
}

pub fn from_yaml(s: &str) -> std::result::Result<EntryOwn, serde_yaml::Error> {
    serde_yaml::from_str(s)
}

pub type Entry<'e> = EntryT<&'e [u8]>;
pub type EntryOwn = EntryT<Box<[u8]>>;

#[derive(Copy, Clone)]
pub struct EntryDB {
    db: Database,
}

impl EntryDB {
    pub fn open(dbm: &DBManager) -> Result<Self> {
         let db = dbm.create_named("entry")?;
         Ok( Self::new(db) )
    }

    pub fn new(db: Database) -> Self {
        Self { db }
    }

    fn get_bytes<'txn, T: Transaction, K: AsRef<[u8]>>(self, txn: &'txn T, key: &K) -> Result<&'txn [u8]> {
        txn.get(self.db, key).map_err(Error::LMDB)
    }

    fn reserve_bytes<'txn, K: AsRef<[u8]>>(self, txn: &'txn mut RwTransaction, key: &K, len: usize, flags: WriteFlags) -> Result<&'txn mut [u8]> {
        txn.reserve(self.db, key, len as size_t, flags).map_err(Error::LMDB)
    }

    pub fn get<'txn, T: Transaction>(self, txn: &'txn T, key: &UUID) -> Result<Entry<'txn>> {
        self.get_bytes(txn, &key.as_bytes()).and_then(Entry::decode)
    }

    pub fn put<'txn, B>(self, txn: &mut RwTransaction, key: &UUID, e: &EntryT<B>) -> Result<()>
        where B: AsRef<[u8]> + Serialize + Deserialize<'txn>
    {
        let len = e.encoded_size()? as usize;
        let buf = self.reserve_bytes(txn, &key.as_bytes(), len, WriteFlags::empty())?;
        e.encode_into(buf)
    }

    pub fn iter_start<'txn, T: Transaction>(self, txn: &'txn T) -> Result<Iter<'txn>> {
        let mut cursor = txn.open_ro_cursor(self.db)?;
        Ok(cursor.iter_start())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuid_cast() {
        let uuid_1 = UUID::generate();
        let uuid_i = uuid_1.as_uuid();
        assert_eq!(uuid_1, UUID::new(uuid_i));
    }
}
