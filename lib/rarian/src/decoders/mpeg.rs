use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::fs::File;
use crate::storage::{MetadataOwned, Metakey};
use crate::error::{Result, Error};
use crate::decoders::DecodeError;
use id3::Tag as Id3Tag;

use core::pin::Pin;
use futures::task;
use futures::stream::Stream;
use futures::Poll;

pub struct MpegDecoder<S> {
    paths: S
}
impl<S: Stream<Item=PathBuf> + Unpin> Stream for MpegDecoder<S> {
    type Item = Result<MetadataOwned>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut task::Context) -> Poll<Option<Self::Item>> {
        if let Some(path) = futures::ready!( Stream::poll_next(Pin::new(&mut self.paths), cx) ) {
            let out = File::open(&path).and_then(|f| {
                match Id3Tag::read_from(&f) {
                    Ok(tag) => {
                        let filename = path.file_name().and_then(OsStr::to_str).map(str::to_string).unwrap();
                        let title = tag.title().unwrap_or_else(|| &filename).to_string();
                        let author = tag.artist().map(|s| s.to_string());
                        let filesize = f.metadata().ok().map(|m| m.len() as usize);

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

                        let m = MetadataOwned::new(title, author, filename, filesize, metamap);

                        Ok(m)
                    }
                    Err(e) => {
                        Err(e.into())
                    }
                }
            });

            return Poll::Ready(Some(out));
        } else {
            return Poll::Ready(None);
        }
    }
}
impl<I> MpegDecoder<I> {
    pub fn new(paths: I) -> Self {
        MpegDecoder { paths }
    }
}

impl From<id3::Error> for DecodeError {
    fn from(e: id3::Error) -> Self {
        DecodeError::Id3(e)
    }
}
