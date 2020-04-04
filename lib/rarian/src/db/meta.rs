use std::collections::{HashMap, HashSet};
use std::fmt;
use std::marker::PhantomData;

use serde::de::{Deserializer, Visitor, MapAccess};

use serde::{Serialize, Deserialize};

use crate::error::{Result, Error};

//pub type Metakey = u32;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Metakey {
    Date,
    Title,
    Description,
    Artist,
}

impl Metakey {
    pub fn from_str(s: &str) -> Result<Metakey> {
        match s {
            "date" => Ok(Metakey::Date),
            "title" => Ok(Metakey::Title),
            "description" => Ok(Metakey::Description),
            "artist" => Ok(Metakey::Artist),
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
