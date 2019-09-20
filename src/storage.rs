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
    Title,
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
}

use std::marker::PhantomData;
pub struct Subject;
impl<'de> Meta<'de> for Subject {
    type Value = &'de str;
}

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

    /// The lifeform, lifeforms or intelligent computer program(s) that created this object.
    author: Vec<&'e str>,

    /// The Filename is relatively often used so we save it as well
    filename: &'e str,

    metamap: HashMap<Metakey, &'e str>,
}
