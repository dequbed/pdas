use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;
use metaflac::Tag as FlacTag;
use std::fs::File;
use crate::storage::{MetadataOwned, Metakey};
use crate::error::Result;
use crate::decoders::DecodeError;

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
                        let author = vc.artist().map(|v| v[0].clone());
                        let filesize = f.metadata().ok().map(|m| m.len() as usize);

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
                            let buf = Box::new(track.to_le_bytes());
                            metamap.insert(Metakey::Track, buf);
                        }
                        if let Some(ttrack) = vc.total_tracks() {
                            let buf = Box::new(ttrack.to_le_bytes());
                            metamap.insert(Metakey::Totaltracks, buf);
                        }
                        if let Some(albumartist) = vc.album_artist() { 
                            let album = albumartist[0].clone();
                            let albuf = album.into_boxed_str().into_boxed_bytes();
                            metamap.insert(Metakey::Albumartist, albuf);
                        }

                        let m = MetadataOwned::new(title, author, filename, filesize, metamap);
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

impl From<metaflac::Error> for DecodeError {
    fn from(e: metaflac::Error) -> Self {
        DecodeError::Metaflac(e)
    }
}

