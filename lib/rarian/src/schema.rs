use serde::{Serialize, Deserialize};

use crate::error::{Result, Error};
use crate::db::meta::Metakey;

use std::hash::Hash;
use std::collections::HashMap;

use crate::db::Index;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum Attributetype {
    Timestamp,
    Unsigned,
    Signed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Database schema description
///
/// The schema contains all information about the construction of a database both and some
/// meta-information for humans like a name and description
/// It also defines what attributes an entry has and what types those attributes are. 
/// Lastly the indices for the db are saved
pub struct Schema {
    /// Human-readable identifier of the database
    pub name: String,

    /// A (short) description of the intended use of the database
    pub description: String,

    /// Version of rarian-lib this database was last opened with. Used for compatability
    pub version: (u32, u32),

    pub attributes: HashMap<Metakey, IndexDescription>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribute {
    pub index: IndexDescription,
}

impl<'a> Schema {
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

    pub fn from_yaml(input: &[u8]) -> Result<Self> {
        let s = serde_yaml::from_slice(input)?;
        Ok(s)
    }
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


// Schema tells us: Field #XYZ has type ABC and identifier DEF. Type ABC defines encoding/decoding
// rules & possible indices.
// Schema then also defines what fields are indexed in which way.
