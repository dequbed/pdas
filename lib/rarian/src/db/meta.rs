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
    Title(Box<str>),
    Artist(Box<str>),
    Date(i64),
    Comment(Box<str>),
    Description(Box<str>),
    Album(Box<str>),
    TrackNumber(i64),
    Albumartist(Box<str>),
}

impl Metavalue {
    pub fn to_int(&self) -> Option<i64> {
        match self {
            Self::Date(i) => Some(*i),
            Self::TrackNumber(i) => Some(*i),
            _ => None
        }
    }

    pub fn to_str(&self) -> Option<&str> {
        match self {
            Self::Title(s) => Some(&s),
            Self::Artist(s) => Some(&s),
            Self::Comment(s) => Some(&s),
            Self::Description(s) => Some(&s),
            Self::Album(s) => Some(&s),
            Self::Albumartist(s) => Some(&s),
            _ => None,
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
