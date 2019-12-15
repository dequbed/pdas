use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::iter::FromIterator;

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

use crate::storage::{Metakey, Meta, metadata_combine};
use crate::error::{Result, Error};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct FileKey {
    bytes: [u8; 32],
    filesize: Option<usize>,
}

impl FileKey {
    pub fn new(bytes: [u8; 32], filesize: Option<usize>) -> Self {
        Self { bytes, filesize }
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryT<B> {
    filekeys: HashSet<FileKey>,
    metadata: HashMap<Metakey, B>,
}
impl<'e, B> EntryT<B>
    where B: Serialize + Deserialize<'e> + AsRef<[u8]>,
{
    pub fn new(filekey: FileKey, metadata: HashMap<Metakey, B>) -> Self {
        let mut set = HashSet::new();
        set.insert(filekey);

        Self::newv(set, metadata)
    }

    pub fn newv(filekeys: HashSet<FileKey>, metadata: HashMap<Metakey, B>) -> Self {
        Self {
            filekeys,
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

    pub fn keys(&self) -> &HashSet<FileKey> {
        &self.filekeys
    }

    pub fn get<T: Meta<'e>>(&'e self) -> Option<T::Value> {
        self.metadata.get(&T::KEY).map(|r: &B| T::decode(r.as_ref()))
    }

    pub fn metadata(&self) -> &HashMap<Metakey, B> {
        &self.metadata
    }
}

pub type Entry<'e> = EntryT<&'e [u8]>;
pub type EntryOwn = EntryT<Box<[u8]>>;

pub fn combine<'e, B>(a: &'e mut EntryT<B>, b: &'e mut EntryT<B>) -> Option<EntryT<B>>
    where B: Serialize + Deserialize<'e> + AsRef<[u8]> + Clone,
{
    if let Some(m) = metadata_combine(a.metadata(), b.metadata()) {
        let allkeys = FromIterator::from_iter(
            a.keys()
             .union(b.keys())
             .map(|x| *x)
            );
        Some(EntryT::newv(allkeys, m))
    } else {
        None
    }
}

#[derive(Copy, Clone)]
pub struct EntryDB {
    db: Database,
}

impl EntryDB {
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

    pub fn put<'txn, B>(self, txn: &'txn mut RwTransaction, key: &UUID, e: EntryT<B>) -> Result<()>
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
    fn combine_keeps_keys_test() {
        let fk1 = FileKey::new([
            0xc6, 0x6d, 0xe9, 0x14, 0xe8, 0xd2, 0x76, 0xa1, 0xc2, 0x35, 0x10, 0x21, 0xf2, 0x84,
            0xeb, 0x01, 0x8c, 0x5a, 0xca, 0x20, 0xc5, 0x5a, 0x5d, 0xf0, 0xad, 0x59, 0x5f, 0x78,
            0x90, 0x67, 0xe8, 0xc6,
        ], Some(1060620));
        let fk2 = FileKey::new([
            0x75, 0x79, 0xd1, 0x34, 0xf6, 0x3b, 0xbc, 0x4d, 0x4f, 0xd7, 0x04, 0xef, 0xe7, 0xf6,
            0xd9, 0x92, 0x31, 0xb0, 0xbc, 0xd9, 0x2d, 0x88, 0x6a, 0x81, 0x6a, 0x83, 0xc4, 0xb4,
            0xf2, 0xd0, 0xbc, 0x26,
        ], None);
        let fk3 = FileKey::new([
            0x95, 0xac, 0x67, 0x8b, 0xaa, 0xcc, 0x67, 0xd3, 0x4f, 0xf8, 0x77, 0x3a, 0x91, 0xbe,
            0xa8, 0xfb, 0xaf, 0xc2, 0x74, 0x80, 0x8c, 0x35, 0xe8, 0xdd, 0x67, 0xfb, 0x03, 0x6d,
            0xc8, 0x68, 0x58, 0x5b,
        ], Some(587978));
        let fk4 = FileKey::new([
            0xe8, 0x6a, 0x00, 0x79, 0x4c, 0x09, 0xf4, 0x63, 0x99, 0xca, 0x0b, 0x13, 0xa3, 0xf8,
            0xc1, 0x04, 0x4d, 0x09, 0x76, 0x9f, 0x77, 0xf1, 0x56, 0x03, 0x66, 0x00, 0x8c, 0xa5,
            0xbc, 0x2b, 0x01, 0xea,
        ], None);
        let fk5 = FileKey::new([
            0xc6, 0x6d, 0xe9, 0x14, 0xe8, 0xd2, 0x76, 0xa1, 0xc2, 0x35, 0x10, 0x21, 0xf2, 0x84,
            0xeb, 0x01, 0x8c, 0x5a, 0xca, 0x20, 0xc5, 0x5a, 0x5d, 0xf0, 0xad, 0x59, 0x5f, 0x78,
            0x90, 0x67, 0xe8, 0xc6,
        ], Some(1060620));
        let fk6 = FileKey::new([
            0x75, 0x79, 0xd1, 0x34, 0xf6, 0x3b, 0xbc, 0x4d, 0x4f, 0xd7, 0x04, 0xef, 0xe7, 0xf6,
            0xd9, 0x92, 0x31, 0xb0, 0xbc, 0xd9, 0x2d, 0x88, 0x6a, 0x81, 0x6a, 0x83, 0xc4, 0xb4,
            0xf2, 0xd0, 0xbc, 0x26,
        ], Some(1202408));
        let fk7 = FileKey::new([
            0x74, 0xe0, 0x56, 0xca, 0x45, 0xf2, 0xe7, 0x07, 0xc7, 0x32, 0x97, 0x9d, 0xd6, 0x8c,
            0xde, 0xe3, 0xef, 0x7a, 0x7e, 0x8f, 0xfc, 0x49, 0xa6, 0xc3, 0x8c, 0x56, 0x9e, 0x37,
            0x9e, 0x21, 0x02, 0x47, 
        ], None);
        let fk8 = FileKey::new([
            0xbe, 0xfe, 0xbf, 0x39, 0x8d, 0xc8, 0xd8, 0x3e, 0xb9, 0x61, 0x5f, 0x1e, 0xf1, 0x62,
            0x94, 0x33, 0x3c, 0x2d, 0xa0, 0x2f, 0xaa, 0x87, 0x51, 0xc4, 0xa5, 0xd4, 0xe7, 0x7e,
            0x80, 0xd9, 0x3b, 0x63, 
        ], Some(676044));

        let m: HashMap<Metakey, &[u8]> = HashMap::new();

        let mut aset = HashSet::new();
        assert!(aset.insert(fk1));
        assert!(aset.insert(fk2));
        assert!(aset.insert(fk3));
        assert!(aset.insert(fk4));
        let mut a = Entry::newv(aset, m.clone());

        let mut bset = HashSet::new();
        assert!(bset.insert(fk5));
        assert!(bset.insert(fk6));
        assert!(bset.insert(fk7));
        assert!(bset.insert(fk8));

        let mut b = Entry::newv(bset, m.clone());

        let mut cset = HashSet::new();
        assert!(cset.insert(fk1));
        assert!(cset.insert(fk2));
        assert!(cset.insert(fk3));
        assert!(cset.insert(fk4));
        assert!(cset.insert(fk6));
        assert!(cset.insert(fk7));
        assert!(cset.insert(fk8));

        assert_eq!(
            combine(&mut a, &mut b),
            Some(Entry::newv(cset, m.clone()))
        )
    }
}
