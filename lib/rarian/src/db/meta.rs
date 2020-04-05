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


pub trait Metavalue<'de> {
    fn decode(bytes: &'de [u8]) -> Self;
}

impl<'de> Metavalue<'de> for &'de str {
    fn decode(bytes: &'de [u8]) -> Self {
        unsafe { std::str::from_utf8_unchecked(bytes) }
    }
}
