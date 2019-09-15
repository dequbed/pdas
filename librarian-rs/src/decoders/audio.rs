use std::path::Path;
use std::fs::File;

use std::ffi::OsStr;

use serde::{Serialize, Deserialize};

use metaflac::Tag;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Song {
    pub artist: Vec<String>,
    pub title: String,
    pub album: Option<String>,
    pub genre: Option<String>,
    pub track: Option<u32>,
    pub totaltracks: Option<u32>,
    pub albumartist: Option<String>,
    pub lyrics: Option<String>,
}

pub struct FlacDecoder;

impl FlacDecoder {
    pub fn decode(paths: &[&Path]) -> Vec<Song> {
        let mut rv = Vec::with_capacity(paths.len());

        for p in paths {
            let mut f = File::open(p).unwrap();
            let tag = Tag::read_from(&mut f);

            match tag {
                Ok(t) => {
                    if let Some(vc) = t.vorbis_comments() {
                        println!("{:?}", vc);
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
