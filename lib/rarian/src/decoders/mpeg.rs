use std::collections::HashMap;
use std::fs::File;
use crate::storage::Metakey;
use crate::error::{Result, Error};
use crate::decoders::DecodeError;
use id3::Tag as Id3Tag;

use core::pin::Pin;
use futures::task;
use futures::stream::Stream;
use futures::task::Poll;

pub struct MpegDecoder;

impl MpegDecoder {
    pub fn new() -> Self {
        Self
    }

    pub fn decode(&self, fp: File) -> Result<HashMap<Metakey, Box<[u8]>>> {
        match Id3Tag::read_from(&fp) {
            Ok(tag) => {
                let mut metamap = HashMap::new();

                if let Some(s) = tag.title() {
                    let buf = s.to_string().into_boxed_str().into_boxed_bytes();
                    metamap.insert(Metakey::Title, buf);
                }
                if let Some(s) = tag.artist() {
                    let buf = s.to_string().into_boxed_str().into_boxed_bytes();
                    metamap.insert(Metakey::Artist, buf);
                }
                if let Some(album) = tag.album() { 
                    let albuf = album.to_string().into_boxed_str().into_boxed_bytes();
                    metamap.insert(Metakey::Album, albuf);
                }
                if let Some(genre) = tag.genre() {
                    let genbuf = genre.to_string().into_boxed_str().into_boxed_bytes();
                    metamap.insert(Metakey::Genre, genbuf);
                }
                if let Some(track) = tag.track() {
                    let buf = Box::new(track.to_le_bytes());
                    metamap.insert(Metakey::Track, buf);
                }
                if let Some(ttrack) = tag.total_tracks() {
                    let buf = Box::new(ttrack.to_le_bytes());
                    metamap.insert(Metakey::Totaltracks, buf);
                }
                if let Some(artist) = tag.album_artist() { 
                    let albuf = artist.to_string().into_boxed_str().into_boxed_bytes();
                    metamap.insert(Metakey::Albumartist, albuf);
                }

                Ok(metamap)
            }
            Err(e) => {
                let e: DecodeError = e.into();
                Err(e.into())
            }
        }
    }
}

impl From<id3::Error> for DecodeError {
    fn from(e: id3::Error) -> Self {
        DecodeError::Id3(e)
    }
}
