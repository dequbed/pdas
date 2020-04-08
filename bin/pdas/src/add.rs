use std::collections::{HashMap, HashSet};

use clap;
use slog::Logger;

use serde::Deserialize;

use rarian::db::entry::EntryOwn;
use rarian::db::meta::Metakey;

use crate::Settings;

pub fn add(log: Logger, s: Settings, m: &clap::ArgMatches) {
}

#[derive(Debug,Deserialize)]
struct Exiftag {
    #[serde(rename = "Title")]
    title: Option<String>,
    #[serde(rename = "Artist")]
    artist: Option<String>,
    #[serde(rename = "Comment")]
    comment: Option<String>,
    #[serde(rename = "Album")]
    album: Option<String>,
    #[serde(rename = "TrackNumber")]
    tracknr: Option<u64>,
    #[serde(rename = "Albumartist")]
    albumartist: Option<String>,
}

fn tagtoentry(tag: Exiftag) -> EntryOwn {
    let mut metadata = HashMap::new();
    if let Some(title) = tag.title {
        metadata.insert(Metakey::Title, title.into_bytes().into_boxed_slice());
    }
    if let Some(artist) = tag.artist {
        metadata.insert(Metakey::Artist, artist.into_bytes().into_boxed_slice());
    }
    if let Some(comment) = tag.comment {
        metadata.insert(Metakey::Comment, comment.into_bytes().into_boxed_slice());
    }
    if let Some(album) = tag.album {
        metadata.insert(Metakey::Album, album.into_bytes().into_boxed_slice());
    }
    if let Some(tracknr) = tag.tracknr {
        metadata.insert(Metakey::TrackNumber, Box::new(tracknr.to_le_bytes()));
    }
    if let Some(albumartist) = tag.albumartist {
        metadata.insert(Metakey::Albumartist, albumartist.into_bytes().into_boxed_slice());
    }

    EntryOwn::newv(HashSet::new(), metadata)
}
