use std::collections::HashMap;

use std::path::PathBuf;

use clap;
use slog::Logger;

use serde::Deserialize;

use rarian::db::Database;
use rarian::db::entry::{EntryT, FileT};
use rarian::db::meta::{Metakey, Metavalue};
use rarian::db::dbm::{self, DBManager};
use rarian::Transaction;

use crate::Settings;

use futures::prelude::*;

pub async fn add(log: &Logger, s: Settings, m: &clap::ArgMatches<'_>) {
    let target = m.value_of("target").expect("No value for `TARGET` set!");
    let files = stream::iter(m.values_of("files").expect("No files provided").map(str::to_string));

    let mut dbmb = DBManager::builder();
    dbmb.set_flags(dbm::EnvironmentFlags::empty());
    dbmb.set_max_dbs(126);
    dbmb.set_map_size(10485760);
    let dbm = DBManager::from_builder(&s.databasepath, dbmb).unwrap();

    let mut txn = dbm.write().unwrap();
    info!(log, "Opening database {}", target);
    let mut db = match Database::open(&txn, target) {
        Ok(db) => db,
        Err(e) => {
            crit!(log, "Can't open database {}: {:?}", target, e);
            return;
        }
    };

    let (f, r) = git_annex::add::add(files);

    if let Err(e) = r {
        error!(log, "Failed to run git-annex add properly: {}", e);
        return;
    }
    let s = r.unwrap();

    let f = f.map(async move |r| if let Err(e) = r {
        error!(log, "annex-poll: {}", e);
    });

    let f2 = s.map(|r| match r {
        Ok((key, file)) => {
            let tag = run_exiftool(file)?;

            let meta = tagtometa(tag);

            let file = FileT { key: key, format: HashMap::new() };

            Ok(EntryT::new(file, meta))
        },
        Err(e) => Err(e)
    }).for_each(|r| {
        match r {
            Ok(entry) => {
                // Check if dupliate
                // Insert entry to db
                if let Err(e) = db.insert_rand(&mut txn, &entry) {
                    error!(log, "Insert Entry: {:?}", e);
                }
            }
            Err(e) => {
                error!(log, "Bad Entry: {}", e);
            }
        }

        future::ready(())
    });

    join!(f.await, f2);

    Transaction::commit(txn);
}

fn run_exiftool(file: String) -> Result<Exiftag, String> {
    use std::process::Command;

    let output = Command::new("exiftool")
        .arg("-j")
        .arg(&file)
        .output()
        .expect("Failed to execute command");

    let mut r: Vec<Exiftag> = serde_json::from_slice(output.stdout.as_slice()).map_err(|e| format!("{:?}", e))?;
    Ok(r.pop().unwrap())
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

    #[serde(rename = "SourceFile")]
    source_file: Option<PathBuf>,
}

fn tagtometa(tag: Exiftag) -> HashMap<Metakey, Metavalue> {
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

    metadata
}
