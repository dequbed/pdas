use std::collections::{HashMap, HashSet};

use serde::{Serialize, Deserialize};

pub type Metakey = u32;

pub trait Metavalue<'de> {
    fn decode(bytes: &'de [u8]) -> Self;
}

impl<'de> Metavalue<'de> for &'de str {
    fn decode(bytes: &'de [u8]) -> Self {
        unsafe { std::str::from_utf8_unchecked(bytes) }
    }
}
