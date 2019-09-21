use std::path::Path;
use std::fs::File;

use std::ffi::OsStr;

use super::DecodeError;

use metaflac::Tag as FlacTag;
use id3::Tag as Id3Tag;

use crate::storage::Song;
use crate::storage::{MetadataOwned, Metakey};

use std::collections::HashMap;

use crate::error::Result;

use std::iter::Iterator;

use std::path::PathBuf;

pub struct FlacDecoder<I> {
    paths: I,
}
// FIXME:
//impl<'tx, I: Iterator<Item=&'tx Path>> Iterator for FlacDecoder<I> {
impl<I: Iterator<Item=PathBuf>> Iterator for FlacDecoder<I> {
    type Item = Result<MetadataOwned>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(p) = self.paths.next() {
            let mut f = File::open(&p).unwrap();
            let tag = FlacTag::read_from(&mut f);

            match tag {
                Ok(t) => {
                    if let Some(vc) = t.vorbis_comments() {
                        let filename = p.file_name().and_then(OsStr::to_str).map(str::to_string).unwrap();
                        let title = vc.title().map(|v| v.get(0).map(|s| s.clone()).unwrap())
                                .unwrap_or_else(|| filename.clone());
                        let author = vc.artist().map(|v| v[0].clone()).unwrap_or_else(|| "Unknown".to_string());
                        let filesize = f.metadata().unwrap().len();

                        let mut metamap = HashMap::new();

                        if let Some(album) = vc.album() { 
                            let album = album[0].clone();
                            let albuf = album.into_boxed_str().into_boxed_bytes();
                            metamap.insert(Metakey::Album, albuf);
                        }
                        if let Some(genre) = vc.genre() {
                            let genre = genre[0].clone();
                            let genbuf = genre.into_boxed_str().into_boxed_bytes();
                            metamap.insert(Metakey::Genre, genbuf);
                        }
                        if let Some(track) = vc.track() {
                            let mut buf = Box::new(track.to_le_bytes());
                            metamap.insert(Metakey::Track, buf);
                        }
                        if let Some(ttrack) = vc.total_tracks() {
                            let mut buf = Box::new(ttrack.to_le_bytes());
                            metamap.insert(Metakey::Totaltracks, buf);
                        }
                        if let Some(albumartist) = vc.album_artist() { 
                            let album = albumartist[0].clone();
                            let albuf = album.into_boxed_str().into_boxed_bytes();
                            metamap.insert(Metakey::Albumartist, albuf);
                        }

                        let m = MetadataOwned::new(title, author, filename, filesize as usize, metamap);
                        return Some(Ok(m));
                    }
                }
                Err(e) => {
                    error!("Failed to read FLAC tag: {}", e);
                    let e: DecodeError = e.into();
                    return Some(Err(e.into()));
                }
            }
        }

        return None;
    }
}
impl<I> FlacDecoder<I> {
    pub fn new(paths: I) -> Self {
        Self { paths }
    }
}

pub struct Id3Decoder<I> {
    paths: I
}
impl<I: Iterator<Item=PathBuf>> Iterator for Id3Decoder<I> {
    type Item = Result<MetadataOwned>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(p) = self.paths.next() {
            let f = File::open(&p).unwrap();

            let tag = Id3Tag::read_from(&f);

            match tag {
                Ok(tag) => {
                    let filename = p.file_name().and_then(OsStr::to_str).map(str::to_string).unwrap();
                    let title = tag.title().unwrap_or_else(|| &filename).to_string();
                    let author = tag.artist().unwrap_or_else(|| "Unknown").to_string();
                    let filesize = f.metadata().unwrap().len();

                    let mut metamap = HashMap::new();

                    if let Some(album) = tag.album() { 
                        let albuf = album.to_string().into_boxed_str().into_boxed_bytes();
                        metamap.insert(Metakey::Album, albuf);
                    }
                    if let Some(genre) = tag.genre() {
                        let genbuf = genre.to_string().into_boxed_str().into_boxed_bytes();
                        metamap.insert(Metakey::Genre, genbuf);
                    }
                    if let Some(track) = tag.track() {
                        let mut buf = Box::new(track.to_le_bytes());
                        metamap.insert(Metakey::Track, buf);
                    }
                    if let Some(ttrack) = tag.total_tracks() {
                        let mut buf = Box::new(ttrack.to_le_bytes());
                        metamap.insert(Metakey::Totaltracks, buf);
                    }
                    if let Some(artist) = tag.album_artist() { 
                        let albuf = artist.to_string().into_boxed_str().into_boxed_bytes();
                        metamap.insert(Metakey::Albumartist, albuf);
                    }

                    let m = MetadataOwned::new(title, author, filename, filesize as usize, metamap);
                    return Some(Ok(m));
                }
                Err(e) => {
                    error!("Failed to read FLAC tag: {}", e);
                    let e: DecodeError = e.into();
                    return Some(Err(e.into()));
                }
            }
        }

        return None;
    }
}
impl<I> Id3Decoder<I> {
    pub fn new(paths: I) -> Self {
        Self { paths }
    }
}
