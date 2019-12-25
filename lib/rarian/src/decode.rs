use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;

use tree_magic as tm;

use futures::channel::mpsc;

use crate::db::entry::EntryOwn;
use crate::storage::Metakey;
use crate::error::{Result, Error};

use crate::decoders::{
    DecodeError,
    MpegDecoder,
};

pub struct Decoder {
    mpeg: MpegDecoder,
}

impl Decoder {
    pub fn new() -> Self {
        let mpeg = MpegDecoder::new();
        Self { mpeg }
    }

    pub fn decode(&self, path: PathBuf) -> Result<HashMap<Metakey, Box<[u8]>>> {
        let mut fp = File::open(&path)?;
        let mut buf = [0; 2048];
        fp.read(&mut buf)?;


        match tm::from_u8(&buf).as_str() {
            "audio/mpeg" => {
                // MP3
                self.mpeg.decode(fp)
            }
            _ => {
                Err(DecodeError::NotFound.into())
            }
        }
    }
}
