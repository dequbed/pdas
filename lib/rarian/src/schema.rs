use serde::{Serialize, Deserialize};

use crate::error::{Result, Error};
use crate::db::meta::Metakey;

use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub indices: HashMap<Metakey, IndexDescription>
}

impl Schema {
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

pub fn from_yaml(s: &[u8]) -> std::result::Result<Schema, serde_yaml::Error> {
    serde_yaml::from_slice(s)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexDescription {
    StemmedTerm {
        dbname: String,
    },
    RangeTree {
        name: String,
    }
}

// Most important information is what kind of matching I want to be able to do.
// Range query, Set queries (is in set, is not in set, is subset/superset of), Text queries (stem
// of word in text, exact match, prox match)
