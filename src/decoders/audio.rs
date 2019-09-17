use std::path::Path;
use std::fs::File;

use std::ffi::OsStr;

use metaflac::Tag as FlacTag;
use id3::Tag as Id3Tag;

use crate::storage::Song;

pub struct FlacDecoder;

impl FlacDecoder {
    pub fn decode(paths: &[&Path]) -> Vec<Song> {
        let mut rv = Vec::with_capacity(paths.len());

        for p in paths {
            let mut f = File::open(p).unwrap();
            let tag = FlacTag::read_from(&mut f);

            match tag {
                Ok(t) => {
                    if let Some(vc) = t.vorbis_comments() {
                        rv.push(Song {
                            artist: vc.artist().map(|v| v.clone()).unwrap_or_else(Vec::new),
                            title: vc.title().map(|v| v.get(0).map(|s| s.clone()))
                                .unwrap_or_else(|| p.file_name().and_then(OsStr::to_str).map(str::to_string)).unwrap(),
                            album: vc.album().map_or(None, |v| v.get(0).map(|s| s.clone())),
                            genre: vc.genre().map_or(None, |v| v.get(0).map(|s| s.clone())),
                            track: vc.track(),
                            totaltracks: vc.total_tracks(),
                            albumartist: vc.album_artist().map_or(None, |v| v.get(0).map(|s| s.clone())),
                            lyrics: vc.lyrics().map_or(None, |v| v.get(0).map(|s| s.clone())),
                        });
                    }
                }
                Err(e) => error!("Failed to read FLAC tag: {}", e),
            }
        }

        rv
    }
}

pub struct Id3Decoder;

impl Id3Decoder {
    pub fn decode(paths: &[&Path]) -> Vec<Song> {
        let mut rv = Vec::with_capacity(paths.len());

        for p in paths {
            match Id3Tag::read_from_path(p) {
                Ok(tag) => {
                    rv.push(Song {
                        artist: tag.artist().map(str::to_string).map(|s| vec![s]).unwrap_or_else(Vec::new),
                        title: tag.title().map(str::to_string)
                                .unwrap_or_else(|| p.file_name().and_then(OsStr::to_str).map(str::to_string).unwrap()),
                        album: tag.album().map(str::to_string),
                        genre: tag.genre().map(str::to_string),
                        track: tag.track(),
                        totaltracks: tag.total_tracks(),
                        albumartist: tag.album_artist().map(str::to_string),
                        // FIXME: ID3 can very much contain lyrics. Different language ones even.
                        lyrics: None,
                    });
                }
                Err(e) => error!("Unable to read ID3 tag for {}", p.display()),
            }
        }

        rv
    }
}
