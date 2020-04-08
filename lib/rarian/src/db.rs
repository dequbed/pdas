pub mod entry;
pub use entry::EntryDB;
pub mod term;
pub use term::TermDB;
pub mod range;
pub use range::RangeDB;
pub mod filekey;
pub use filekey::FilekeyDB;

pub mod dbm;
pub mod meta;

use entry::EntryT;
use crate::error::{Result, Error};
pub use crate::uuid::UUID;
use crate::schema::{Schema, IndexDescription};
use dbm::DBManager;

use std::collections::HashMap;
use std::convert::TryInto;

use lmdb::{Transaction, RoTransaction, RwTransaction};

use serde::{
    Deserialize,
    Serialize,
};

use std::path::Path;
use std::fs::{self, File};
use std::io::{Read, Write};

/// Main DB type, keeps track of entries and indices
pub struct Database {
    pub entries: EntryDB,
    pub filekeys: FilekeyDB,
    pub indices: HashMap<meta::Metakey, Index>,
}

impl<'env> Database {
    fn new(entries: EntryDB, indices: HashMap<meta::Metakey, Index>, filekeys: FilekeyDB) -> Self {
        Self { entries, indices, filekeys }
    }

    pub fn open<T: Transaction>(txn: &T, roname: &str) -> Result<Self> {
        let mut name = roname.to_string();
        let len = name.len();

        let db = unsafe { txn.open_db(None)? };
        name.push_str("_schema");

        let b = txn.get(db, &name.as_bytes())?;
        let schema = Schema::decode(b)?;

        name.replace_range(len.., "_filekeys");
        let db = unsafe { txn.open_db(Some(&name))? };
        let filekeys = FilekeyDB::new(db);

        let indices: HashMap<meta::Metakey, Index> = schema.attributes.iter()
            .filter_map(|(k,a)| Index::construct(txn, db, a).ok().map(|x| (*k,x)))
            .collect();

        let entries = unsafe { txn.open_db(Some(roname))? };
        let entries = EntryDB::new(entries);

        Ok(Self::new(entries, indices, filekeys))
    }

    pub fn create(txn: &mut RwTransaction, roname: &str, schema: Schema) -> Result<()> {
        let mut name = roname.to_string();
        let len = name.len();

        let db = unsafe {
            txn.open_db(None)?
        };
        name.push_str("_schema");

        let schema_size = schema.encoded_size()? as usize;
        let schema_buf = txn.reserve(db, &name.as_bytes(), schema_size, lmdb::WriteFlags::empty())?;
        schema.encode_into(schema_buf)?;

        name.replace_range(len.., "_filekeys");
        unsafe {
            txn.create_db(Some(&name), lmdb::DatabaseFlags::empty())?;
        }

        for (k, index) in schema.attributes.iter() {
            println!("Creating index for {:?}", k);
            Index::create(txn, db, index).ok();
            println!("index {:?} created", k);
        }

        unsafe {
            txn.create_db(Some(roname), lmdb::DatabaseFlags::empty())?;
        }

        Ok(())
    }

    pub fn insert(&mut self, txn: &mut RwTransaction, uuid: &UUID, entry: &EntryT) -> Result<()>
    {
        // 1: Index entry
        for (key, i) in self.indices.iter_mut() {
            if let Some(val) = entry.metadata.get(key) {
                i.index(txn, *uuid, val)?;
            }
        }

        // 2: Insert into entry & filkey db
        self.entries.put(txn, uuid, entry)?;
        for file in entry.files.iter() {
            self.filekeys.put(txn, &file.key, uuid)?;
        }

        Ok(())
    }

    pub fn dump(&self, txn: &RoTransaction) -> Result<()> {
        self.entries.list(txn)?;
        println!("Indices:\n==============================");
        for (k, db) in self.indices.iter() {
            println!("{:?}:\n", k);
            db.list(txn)?;
        }
        Ok(())
    }

    pub fn export_with<'txn, T: Transaction>(&self, dir: &Path, txn: &'txn T) -> Result<()> {
        let i = self.entries.iter_start(txn)?;

        for r in i {
            if let Ok((k,v)) = r {
                let e = entry::EntryT::decode(v)?;
                let u = {
                    let (int_bytes, _rest) = k.split_at(std::mem::size_of::<u128>());
                    // This can fail if for some reason entrydb keys are less than 16 bytes long.
                    // In that case we don't have any idea how to handle or export that entry. Just
                    // give up.
                    let u = u128::from_le_bytes(int_bytes.try_into().unwrap());

                    UUID::from_u128(u)
                };

                let mut p = Path::join(&dir, "entries/");
                fs::create_dir_all(&p)?;
                p.push(format!("{}.yaml", u.as_uuid()));
                let mut fp = File::create(&p)?;
                let s = e.to_yaml()?;
                println!("Writing file: {:?}", &p);
                fp.write_all(s.as_ref())?;
            }
        }

        Ok(())
    }

    pub fn import(&mut self, txn: &mut RwTransaction, dir: &Path) -> Result<()> {
        let dir = dir.join("entries/");
        println!("Reading dir: {:?}", dir);
        let entries = fs::read_dir(dir)?;


        let i = entries
            .filter_map(std::result::Result::ok)
            .filter(|d| {
                if let Ok(true) = d.file_type().and_then(|f| Ok(f.is_file())) {
                    return true;
                }
                return false;
            })
            .map(|d| d.path());

        for path in i {
            if let Some(uuid_str) = path.file_stem().and_then(|os| os.to_str()) {
                let u = UUID::parse_str(uuid_str)?;
                let mut fp = File::open(path)?;
                let mut buf = Vec::new();
                fp.read_to_end(&mut buf)?;
                let e = entry::from_yaml(&buf)?;

                self.insert(txn, &u, &e)?;

                println!("Imported {}", u.as_uuid());
            }
        }

        Ok(())
    }
}

#[derive(Debug,Clone)]
pub enum Index {
    IntMap(RangeDB),
    Term(TermDB),
}

impl Index {
    #[inline]
    pub fn index(&mut self, txn: &mut RwTransaction, uuid: UUID, entry_v: &meta::Metavalue) -> Result<()> {
        match self {
            Self::IntMap(db) => {
                let value = entry_v.to_int().ok_or(Error::TypeError)?;
                db.index(txn, value, uuid)
            },
            Self::Term(db) => {
                let term = entry_v.to_str().ok_or(Error::TypeError)?;
                db.index(txn, term.to_string(), uuid)
            }
        }
    }

    #[inline]
    pub fn construct<'txn, T: Transaction> (txn: &'txn T, db: lmdb::Database, desc: &IndexDescription) 
        -> Result<Self> 
    {
        match desc {
            IndexDescription::RangeTree { name } => {
                let bytes = txn.get(db, name)?;
                let map = RangeDB::decode(bytes)?;
                Ok(Self::IntMap(RangeDB::new(db, name.clone(), map)))
            },
            IndexDescription::StemmedTerm { dbname } => {
                let db = unsafe { txn.open_db(Some(dbname))? };
                Ok(Self::Term(TermDB::new(db)))
            }
        }
    }

    #[inline]
    pub fn create(txn: &mut RwTransaction, db: lmdb::Database, desc: &IndexDescription) 
        -> Result<()>
    {
        match desc {
            IndexDescription::RangeTree { name } => {
                let buf = txn.reserve(db, name, RangeDB::empty_encoded_size()? as usize, 
                    lmdb::WriteFlags::empty())?;
                RangeDB::empty_encode_into(buf)?;
                Ok(())
            },
            IndexDescription::StemmedTerm { dbname } => {
                unsafe {
                    txn.create_db(Some(dbname), lmdb::DatabaseFlags::empty())?;
                }
                Ok(())
            }
        }
    }

    pub fn list<'txn, T: Transaction>(&self, txn: &'txn T) -> Result<()> {
        match self {
            Self::IntMap(db) => {
                db.list()
            },
            Self::Term(db) => {
                db.list(txn)
            }
        }
    }
}
