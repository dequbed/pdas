use std::collections::HashMap;

use serde::{Serialize, Deserialize};


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Book {
    pub filename: String,
    pub author: Option<String>,
    pub title: Option<String>,
    pub subject: Option<String>,
    pub description: Option<String>,
    pub date: Option<String>,
    pub identifier: Option<String>,
    pub language: Option<String>,
    pub publisher: Option<String>,
    pub license: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Song {
    pub artist: Vec<String>,
    pub title: String,
    pub album: Option<String>,
    pub genre: Option<String>,
    pub track: Option<u32>,
    pub totaltracks: Option<u32>,
    pub albumartist: Option<String>,
    pub lyrics: Option<String>,
}


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Storables {
    Text(Book),
    Audio(Song),
}

impl Storables {
    pub fn title(&self) -> String {
        match self {
            Storables::Text(b) => match b.title { 
                Some(ref b) => b.clone(),
                None => b.filename.clone(),
            },
            Storables::Audio(s) => s.title.clone(),
        }
    }
}

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

trait Meta<'de> {
    // This way different Metadata can have different Rust representations
    type Value: Serialize + Deserialize<'de>;
    const KEY: Metakey;
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

    fn decode(bytes: &'de [u8]) -> Self::Value;
}

use std::marker::PhantomData;
pub struct Subject;
impl<'de> Meta<'de> for Subject {
    type Value = &'de str;

    const KEY: Metakey = Metakey::Subject;

    fn decode(bytes: &'de [u8]) -> Self::Value {
        unsafe { std::str::from_utf8_unchecked(bytes) }
    }
}

// Decoding HashMap<MetaKey, Value>:
// let (key, offset, len) = header.decode_next_key();
// match key {
//      Metakey::Subject => Subject::decode(&values[offset..len]),
//      [...]
// }

// To do this well we will need to implement Serialize/Deserialize by hand. Not too much work
// though

// NOTICE: This structure should always be READ-optimized. Heavy memcpy for writes is acceptable,
// but reading must not need to copy or do expensive decoding operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metadata<'e> {
    /// A human-readable identifier for this object. The tile will be tokenized and indexed, so
    /// it may contain several words.
    /// It should not contain redundant information, e.g. name the author when the 'author' field
    /// is already set.
    title: &'e str,

    /// The lifeform or intelligent computer program that created this object.
    author: &'e str,

    /// The Filename is relatively often used so we save it as well
    filename: &'e str,

    metamap: HashMap<Metakey, &'e [u8]>,
}

use crate::error::Error;

impl<'e> Metadata<'e> {
    pub fn new(title: &'e str, author: &'e str, filename: &'e str, metamap: HashMap<Metakey, &'e [u8]>) -> Self {
        Self {
            title, author, filename, metamap
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
    pub fn get<T: Meta<'e>>(&self) -> Option<T::Value> {
        self.metamap.get(&T::KEY).map(|r| T::decode(*r))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_metadata_test() {
        let title = "testtitle";
        let author = "testauthor";
        let filename = "testfilename";
        let subject = "testsubject";
        let mut metamap = HashMap::new();
        metamap.insert(Metakey::Subject, subject.as_bytes());
        let m = Metadata::new(title, author, filename, metamap);
        println!("{:?}", m);

        let l = m.encoded_size().unwrap() as usize;
        let mut vec = Vec::with_capacity(l);
        unsafe { vec.set_len(l) };
        m.encode_into(&mut vec[..l]).unwrap();

        let n = Metadata::decode(&vec[..l]).unwrap();
        assert_eq!(m,n);
        assert_eq!(n.get::<Subject>(), Some(subject));
    }
}
