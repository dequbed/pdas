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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metadata {
    /// A human-readable identifier for this object. The tile will be tokenized and indexed, so
    /// it may contain several words.
    /// It should not contain redundant information, e.g. name the author when the 'author' field
    /// is already set.
    title: String,

    /// The lifeform, lifeforms or intelligent computer program(s) that created this object.
    author: Vec<String>,

    /// The Filename is relatively often used so we save it as well
    filename: String,

    metamap: HashMap<Metakey, String>,
}
