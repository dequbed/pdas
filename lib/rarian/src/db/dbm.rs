use crate::error::{Result, Error};
use serde::{Serialize, Deserialize};
use libc::size_t;
use std::fmt;
use std::collections::HashMap;
use rust_stemmers::{Algorithm, Stemmer};

pub use lmdb::{
    Environment,
    EnvironmentFlags,
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

    pub fn open(&self) -> Result<lmdb::Database> {
        self.env.open_db(None).map_err(Error::LMDB)
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
