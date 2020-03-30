use std::iter::Iterator;
use std::ops::RangeBounds;
use std::collections::BTreeMap;

use serde::{
    Deserialize,
    Serialize,
};

use crate::uuid::UUID;
use crate::error::{Result, Error};

#[derive(Debug, Clone)]
pub struct RangeDB {
    db: lmdb::Database,
    name: String,
    map: BTreeMap<u64, UUID>,
}

impl RangeDB {
    pub fn new(db: lmdb::Database, name: String, map: BTreeMap<u64, UUID>) -> Self {
        Self { db, name, map }
    }
    pub fn range<R: RangeBounds<u64>>(&self, r: R) -> impl Iterator<Item = (&u64, &UUID)> {
        self.map.range(r)
    }

    pub fn decode(bytes: &[u8]) -> Result<BTreeMap<u64, UUID>> {
        bincode::deserialize(bytes).map_err(Error::Bincode)
    }

    pub fn encode_into(&self, bytes: &mut [u8]) -> Result<()> {
        bincode::serialize_into(bytes, &self.map).map_err(Error::Bincode)
    }

    pub fn encoded_size(&self) -> Result<u64> {
        bincode::serialized_size(&self.map).map_err(Error::Bincode)
    }

    pub fn empty_encoded_size() -> Result<u64> {
        bincode::serialized_size(&BTreeMap::<u64, UUID>::new()).map_err(Error::Bincode)
    }

    pub fn empty_encode_into(bytes: &mut [u8]) -> Result<()> {
        bincode::serialize_into(bytes, &BTreeMap::<u64, UUID>::new()).map_err(Error::Bincode)
    }

    pub fn index(&mut self, txn: &mut lmdb::RwTransaction, value: u64, uuid: UUID) -> Result<()> {
        self.map.insert(value, uuid);
        let size = self.encoded_size()? as usize;
        let bytes = txn.reserve(self.db, &self.name.as_bytes(), size, lmdb::WriteFlags::empty())?;
        self.encode_into(bytes)
    }

    pub fn list(&self) -> Result<()> {
        for (v,u) in self.map.iter() {
            println!("{}:\t{}", v, u.as_uuid())
        }
        Ok(())
    }
}
