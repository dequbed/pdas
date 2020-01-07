use crate::error::{Result, Error};
use super::meta::{Metadata, MetadataS, MetadataOwned};
use serde::{Serialize, Deserialize};
use libc::size_t;
use std::fmt;
use std::collections::HashMap;
use rust_stemmers::{Algorithm, Stemmer};

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

pub struct DBManager {
    env: Environment
}

impl DBManager {
    pub fn builder() -> EnvironmentBuilder {
        Environment::new()
    }

    pub fn from_builder(path: &Path, env: EnvironmentBuilder) -> Result<Self> {
        Ok(DBManager {
            env: env.open(path).map_err(Error::LMDB)?
        })
    }

    pub fn open_named(&self, name: &str) -> Result<lmdb::Database> {
        self.env.open_db(Some(name)).map_err(Error::LMDB)
    }

    pub fn create_named(&self, name: &str) -> Result<lmdb::Database> {
        self.env.create_db(Some(name), DatabaseFlags::empty()).map_err(Error::LMDB)
    }
    pub fn create_named_flags(&self, name: &str, flags: DatabaseFlags) -> Result<lmdb::Database> {
        self.env.create_db(Some(name), flags).map_err(Error::LMDB)
    }

    pub fn read(&self) -> Result<RoTransaction> {
        self.env.begin_ro_txn().map_err(Error::LMDB)
    }

    pub fn write(&self) -> Result<RwTransaction> {
        self.env.begin_rw_txn().map_err(Error::LMDB)
    }
}

/// Keytype for the Metadatabase
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
pub struct SHA256E([u8; 32]);
impl AsRef<[u8]> for SHA256E {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
impl fmt::Debug for SHA256E {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SHA256E--")?;
        for byte in self.0.iter() {
            write!(f, "{:x}", byte)?;
        }
        Ok(())
    }
}
impl SHA256E {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

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
        b'A'..=b'F' => c - b'A' + 10,
        b'a'..=b'f' => c - b'a' + 10,
        b'0'..=b'9' => c - b'0',
        _ => 0
    }
}


/// The Key used to reference a Metadata object
pub type Key = SHA256E;
