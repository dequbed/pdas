use std::collections::HashMap;

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Metakey {
    Subject,
    Description,
    Date,
    Identifier,
    Language,
    Publisher,
    License,
    Album,
    Genre,
    Track,
    Totaltracks,
    Albumartist,
    Lyrics,
}

//            v defined by Metavalue<typeof Value>
// Value -> (Tag, &[u8])
// (Tag, &[u8]) -> Value

pub trait Meta<'de> {
    // This way different Metadata can have different Rust representations
    type Value: Metavalue<'de>;
    // To implement this properly we need to use `Bytes`-based abstractions; a Metatag is pretty
    // much only a specific block of bytes in the larger block of bytes the DB stores for us at
    // that key. When we have that represented we can encode the Metadata by storing the required
    // Metadata up front, then an array of what metavalues are stored (i.e. store `[(Tag, Length)]`
    // specifically so we can read that table without having to decode any of the actual values
    // (very useful in the cases where we want to access only a select few of the values, e.g. when
    // reindexing or printing an object with a view filter) and then finally store a CRC checksum
    // or something similar to allow for error checking.
    //
    // When you want to implement a new Metatag you add
    // ```rust
    // struct Metatag;
    // impl Meta for Metatag {
    //     type Value = u64;
    // }
    // ```
    // Only issue to be solved is how to tag which value has which key. One option would be to
    // simply have a `const ID: u32` that are manually assigned, the other one would be to have an
    // enum with some ordering and use that.
    // One goal is that if new metadata types are added later on the application can still read
    // tags from previous versions. In the case of the tag that means that a value should keep it's
    // tag once it has been assigned

    // For now we're going with the enum variant:
    const KEY: Metakey;

    #[inline(always)]
    fn decode(bytes: &'de [u8]) -> Self::Value {
        Self::Value::decode(bytes)
    }
}

pub trait Metavalue<'de> {
    fn decode(bytes: &'de [u8]) -> Self;
}

impl<'de> Metavalue<'de> for &'de str {
    fn decode(bytes: &'de [u8]) -> Self {
        unsafe { std::str::from_utf8_unchecked(bytes) }
    }
}

pub struct Subject;
impl<'de> Meta<'de> for Subject {
    type Value = &'de str;
    const KEY: Metakey = Metakey::Subject;
}

pub struct Description;
impl<'de> Meta<'de> for Description {
    type Value = &'de str;
    const KEY: Metakey = Metakey::Description;
}

use chrono::{DateTime, Utc};
impl<'de> Metavalue<'de> for DateTime<Utc> {
    fn decode(bytes: &'de [u8]) -> Self {
        bincode::deserialize(bytes).unwrap()
    }
}
pub struct Date;
impl<'de> Meta<'de> for Date {
    type Value = DateTime<Utc>;
    const KEY: Metakey = Metakey::Date;
}

pub struct Identifier;
impl<'de> Meta<'de> for Identifier {
    type Value = &'de str;
    const KEY: Metakey = Metakey::Identifier;
}

// NOTICE: This structure should always be READ-optimized. Heavy memcpy for writes is acceptable,
// but reading must not need to copy or do expensive decoding operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetadataS<S, B> {
    /// A human-readable identifier for this object. The tile will be tokenized and indexed, so
    /// it may contain several words.
    /// It should not contain redundant information, e.g. name the author when the 'author' field
    /// is already set.
    pub title: S,

    /// The lifeform or intelligent computer program that created this object.
    // Sadly we don't always have an author in the metadata
    pub author: Option<S>,

    /// The Filename is relatively often used so we save it as well
    pub filename: S,


    /// the size in bytes of the object this data belongs to
    pub filesize: Option<usize>,

    metamap: HashMap<Metakey, B>,
}

use crate::error::Error;

impl<'e, S, B> MetadataS<S, B> 
    where S: Serialize + Deserialize<'e> + AsRef<str>,
          B: Serialize + Deserialize<'e> + AsRef<[u8]>,
{
    pub fn new(title: S, author: Option<S>, filename: S, filesize: Option<usize>, metamap: HashMap<Metakey, B>) -> Self {
        Self {
            title, author, filename, filesize, metamap
        }
    }

    #[inline(always)]
    pub fn decode(bytes: &'e [u8]) -> Result<Self, Error> {
        bincode::deserialize(bytes).map_err(Error::Bincode)
    }

    #[inline(always)]
    pub fn encode_into(&self, bytes: &mut [u8]) -> Result<(), Error> {
        bincode::serialize_into(bytes, &self).map_err(Error::Bincode)
    }

    #[inline(always)]
    pub fn encoded_size(&self) -> Result<u64, Error> {
        bincode::serialized_size(self).map_err(Error::Bincode)
    }

    #[inline(always)]
    pub fn get<T: Meta<'e>>(&'e self) -> Option<T::Value> {
        self.metamap.get(&T::KEY).map(|r: &B| T::decode(r.as_ref()))
    }
}

pub type Metadata<'e> = MetadataS<&'e str, &'e [u8]>;
pub type MetadataOwned = MetadataS<String, Box<[u8]>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_metadata_test() {
        let title = "testtitle".to_string();
        let author = "testauthor".to_string();
        let filename = "testfilename".to_string();
        let subject = "testsubject".to_string().into_boxed_str().into_boxed_bytes();
        let filesize = 361567;
        let mut metamap = HashMap::new();
        metamap.insert(Metakey::Subject, subject);
        let m = MetadataOwned::new(title, Some(author), filename, Some(filesize), metamap);
        println!("{:?}", m);

        let l = m.encoded_size().unwrap() as usize;
        let mut vec: Vec<u8> = Vec::with_capacity(l);
        unsafe { vec.set_len(l) };
        let mut vec = vec.into_boxed_slice();
        m.encode_into(&mut vec[..l]).unwrap();

        let n = Metadata::decode(&vec[..l]).unwrap();

        assert_eq!(n.title, m.title);
        assert_eq!(n.author.map(|s| s.to_string()), m.author);
        assert_eq!(n.filename, m.filename);
        assert_eq!(n.filesize, m.filesize);
    }
}
