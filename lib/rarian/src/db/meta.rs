use std::collections::{HashMap, HashSet};
use std::fmt;
use std::marker::PhantomData;

use serde::de::{Deserializer, Visitor, MapAccess};

use serde::{Serialize, Deserialize};

use crate::error::{Result, Error};

//pub type Metakey = u32;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Metakey {
    Title,
    Artist,
    Date,
    Comment,
    Description,
    Album,
    TrackNumber,
    Albumartist,
}

impl Metakey {
    pub fn from_str(s: &str) -> Result<Metakey> {
        match s {
            "title" => Ok(Metakey::Title),
            "artist" => Ok(Metakey::Artist),
            "date" => Ok(Metakey::Date),
            "comment" => Ok(Metakey::Comment),
            "description" => Ok(Metakey::Description),
            "album" => Ok(Metakey::Album),
            "tracknumber" => Ok(Metakey::TrackNumber),
            "albumartist" => Ok(Metakey::Albumartist),
            _ => Err(Error::BadMetakey)
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Metavalue {
    Title(Box<[Box<str>]>),
    Artist(Box<[Box<str>]>),
    Date(Box<[i64]>),
    Comment(Box<[Box<str>]>),
    Description(Box<[Box<str>]>),
    Album(Box<[Box<str>]>),
    TrackNumber(Box<[i64]>),
    Albumartist(Box<[Box<str>]>),
}

impl Metavalue {
    pub fn to_int(&self) -> impl Iterator<Item=&i64> {
        match self {
            Self::Date(i) => i.iter(),
            Self::TrackNumber(i) => i.iter(),
            _ => [].iter(),
        }
    }

    pub fn to_str(&self) -> impl Iterator<Item=&Box<str>> {
        match self {
            Self::Title(s) => s.iter(),
            Self::Artist(s) => s.iter(),
            Self::Comment(s) => s.iter(),
            Self::Description(s) => s.iter(),
            Self::Album(s) => s.iter(),
            Self::Albumartist(s) => s.iter(),
            _ => [].iter(),
        }
    }

    pub fn to_key(&self) -> Metakey {
        match self {
            Self::Title(_) => Metakey::Title,
            Self::Artist(_) => Metakey::Artist,
            Self::Date(_) => Metakey::Date,
            Self::Comment(_) => Metakey::Comment,
            Self::Description(_) => Metakey::Description,
            Self::Album(_) => Metakey::Album,
            Self::TrackNumber(_) => Metakey::TrackNumber,
            Self::Albumartist(_) => Metakey::Albumartist,
        }
    }
}

impl fmt::Display for Metavalue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
