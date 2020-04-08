use std::collections::{HashMap, HashSet};

use clap;
use slog::Logger;

use serde::Deserialize;

use rarian::db::entry::EntryOwn;
use rarian::db::meta::{Metakey, Metavalue};

use crate::Settings;

pub fn add(log: Logger, s: Settings, m: &clap::ArgMatches) {
}

#[derive(Debug,Deserialize)]
struct Exiftag {
    #[serde(rename = "Title")]
    title: Option<Box<str>>,
    #[serde(rename = "Artist")]
    artist: Option<Box<str>>,
    #[serde(rename = "Comment")]
    comment: Option<Box<str>>,
    #[serde(rename = "Album")]
    album: Option<Box<str>>,
    #[serde(rename = "TrackNumber")]
    tracknr: Option<i64>,
    #[serde(rename = "Albumartist")]
    albumartist: Option<Box<str>>,
}

fn tagtoentry(tag: Exiftag) -> EntryOwn {
    let mut metadata = HashMap::new();
    if let Some(title) = tag.title {
        metadata.insert(Metakey::Title, Metavalue::Title(title));
    }
    if let Some(artist) = tag.artist {
        metadata.insert(Metakey::Artist, Metavalue::Artist(artist));
    }
    if let Some(comment) = tag.comment {
        metadata.insert(Metakey::Comment, Metavalue::Comment(comment));
    }
    if let Some(album) = tag.album {
        metadata.insert(Metakey::Album, Metavalue::Album(album));
    }
    if let Some(tracknr) = tag.tracknr {
        metadata.insert(Metakey::TrackNumber, Metavalue::TrackNumber(tracknr));
    }
    if let Some(albumartist) = tag.albumartist {
        metadata.insert(Metakey::Albumartist, Metavalue::Albumartist(albumartist));
    }

    EntryOwn::newv(HashSet::new(), metadata)
}
