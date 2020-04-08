use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::iter::FromIterator;

use std::fmt;

use std::fs::{self, File};
use std::io::{Read, Write};
use std::convert::TryInto;
use std::path::Path;
use std::hash::{Hash, Hasher};

use bytes::{Bytes, BytesMut};

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
    Serializer,
    Deserializer,
    ser::SerializeSeq,
};

use libc::size_t;

use crate::db::dbm::DBManager;

use crate::db::meta::{Metakey, Metavalue};
use crate::error::{Result, Error};
use crate::uuid::{UUID, Uuid};

pub type FileKey = String;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum FormatKey {
    /// MIME type of the given file
    MimeType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// A file indexed by git-annex
///
/// Important: This struct has custom `Eq` and `Hash` behaviour in that only the key will be
/// considered, format metadata is ignored.
pub struct FileT {
    pub key: FileKey,
    pub format: HashMap<FormatKey, Box<str>>,
}
impl PartialEq for FileT {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}
impl Eq for FileT {}
impl Hash for FileT {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

impl FileT {
    pub fn new(key: FileKey, format: HashMap<FormatKey, Box<str>>) -> Self {
        Self { key, format }
    }
}

impl FileT {
    pub fn ref_eq(&self, other: &FileT) -> bool
    {
        self.key == other.key &&
            std::iter::Iterator::eq(
                self.format.iter().map(|(k,v)| (k, v.as_ref())), 
                other.format.iter().map(|(k,v)| (k, v.as_ref())))
    }
}

impl fmt::Display for FileT {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "File {}", self.key)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
/// A metadata entry
///
/// Each entry connects a metadata map to a set of filekeys. All filekey should identifiy the
/// semantically same file although its actual bits may be different. For example the same song
/// encoded with FLAC and ogg/vorbis should be the same entry, and the same song but with
/// Vorbis comments or ID3 tags attached / not attached should be the same entry.
pub struct EntryT {
    pub files: HashSet<FileT>,
    /// Metadata is an arbitrary key-value map
    #[serde(serialize_with = "map_to_list", deserialize_with = "list_to_map") ]
    pub metadata: HashMap<Metakey, Metavalue>,
}
impl EntryT {
    pub fn new(filekey: FileT, metadata: HashMap<Metakey, Metavalue>) -> Self {
        let mut set = HashSet::new();
        set.insert(filekey);

        Self::newv(set, metadata)
    }

    pub fn newv(files: HashSet<FileT>, metadata: HashMap<Metakey, Metavalue>) -> Self {
        Self {
            files,
            metadata,
        }
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes).map_err(Error::Bincode)
    }

    pub fn encode_into(&self, bytes: &mut [u8]) -> Result<()> {
        bincode::serialize_into(bytes, &self).map_err(Error::Bincode)
    }

    pub fn encoded_size(&self) -> Result<u64> {
        bincode::serialized_size(self).map_err(Error::Bincode)
    }

    pub fn to_yaml(&self) -> std::result::Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }
}

pub fn from_yaml(s: &[u8]) -> std::result::Result<EntryT, serde_yaml::Error> {
    serde_yaml::from_slice(s)
}

impl fmt::Display for EntryT {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Entry\n")?;
        for file in self.files.iter() {
            write!(f, "\t\t{}\n", file)?;
        }
        write!(f, "\tMetadata:\n")?;
        for meta in self.metadata.values() {
            write!(f, "\t\t{}\n", meta)?;
        }

        Ok(())
    }
}

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

    pub fn put<'txn>(self, txn: &mut RwTransaction, key: &UUID, e: &EntryT) -> Result<()>
    {
        let len = e.encoded_size()? as usize;
        let buf = self.reserve_bytes(txn, &key.as_bytes(), len, WriteFlags::empty())?;
        e.encode_into(buf)
    }

    pub fn get<'txn, T: Transaction>(self, txn: &'txn T, key: &UUID) -> Result<EntryT> {
        self.get_bytes(txn, &key.as_bytes()).and_then(EntryT::decode)
    }

    pub fn iter_start<'txn, T: Transaction>(self, txn: &'txn T) -> Result<Iter<'txn>> {
        let mut cursor = txn.open_ro_cursor(self.db)?;
        Ok(cursor.iter_start())
    }

    pub fn list<'txn, T: Transaction>(&self, txn: &'txn T) -> Result<()> {
        let i = self.iter_start(txn)?;

        for r in i {
            if let Ok((k,v)) = r {
                let e = EntryT::decode(v)?;
                let u = {
                    let (int_bytes, _rest) = k.split_at(std::mem::size_of::<u128>());
                    // This can fail if for some reason entrydb keys are less than 16 bytes long.
                    // In that case we don't have any idea how to handle or export that entry. Just
                    // give up.
                    let u = u128::from_le_bytes(int_bytes.try_into().unwrap());

                    UUID::from_u128(u)
                };
                println!("{}:\t{:?}", u.as_uuid(), e);
            }
        }

        Ok(())
    }
}

fn list_to_map<'de, D: Deserializer<'de>>(deserializer: D) -> std::result::Result<HashMap<Metakey, Metavalue>, D::Error> {
    let mut map = HashMap::new();
    let v = Vec::<Metavalue>::deserialize(deserializer)?;
    for value in v {
        map.insert(value.to_key(), value);
    }

    Ok(map)
}
fn map_to_list<S: Serializer>(map: &HashMap<Metakey, Metavalue>, serializer: S) -> std::result::Result<S::Ok, S::Error>{
    let list: Vec<Metavalue> = map.values().map(|v| v.clone()).collect();
    let mut seq = serializer.serialize_seq(Some(list.len()))?;
    for element in list.iter() {
        seq.serialize_element(&element)?;
    }
    seq.end()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode() {
        let mut files = HashSet::new();
        let mut format = HashMap::new();
        format.insert(FormatKey::MimeType, "audio/flac".to_string().into_boxed_str());
        files.insert(FileT {
            key: "SHA256E-s5338457--d2d5872da46b4a70bda0de855a0d5250bb01e89d52b5e751f1fc685ee4e064f2.flac".to_string(),
            format: format,
        });

        let mut metadata = HashMap::new();
        metadata.insert(Metakey::Title, Metavalue::Title("Leviathan".to_string().into_boxed_str()));
        metadata.insert(Metakey::Artist, Metavalue::Artist("blinch".to_string().into_boxed_str()));
        metadata.insert(Metakey::TrackNumber, Metavalue::TrackNumber(20));

        let e = EntryT::newv(files, metadata);

        let ymlstr = e.to_yaml().expect("Failed to *encode*");
        let e2 = from_yaml(ymlstr.as_bytes()).expect("Failed to *decode*");

        assert_eq!(e, e2);
    }
}
