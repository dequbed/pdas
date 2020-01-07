use std::collections::{HashMap, HashSet};

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Metakey {
    Title,
    Author,
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
    Artist,
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

pub struct Title;
impl<'de> Meta<'de> for Title {
    type Value = &'de str;
    const KEY: Metakey = Metakey::Title;
}

pub struct Author;
impl<'de> Meta<'de> for Author {
    type Value = &'de str;
    const KEY: Metakey = Metakey::Author;
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

pub struct Artist;
impl<'de> Meta<'de> for Artist {
    type Value = &'de str;
    const KEY: Metakey = Metakey::Artist;
}

pub fn metadata_combine<'e, B>(a: &'e HashMap<Metakey, B>, b: &'e HashMap<Metakey, B>)
    -> Option<HashMap<Metakey, B>>
    where B: Serialize + Deserialize<'e> + AsRef<[u8]> + Clone
{
    Some(a.clone())
}

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
