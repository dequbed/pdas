use std::collections::HashMap;
use std::io::{self, BufRead};
use std::path::PathBuf;

use clap;
use slog::Logger;

use serde::Deserialize;

use rarian::db::Database;
use rarian::db::entry::{EntryT, FileT, FormatKey};
use rarian::db::meta::{Metakey, Metavalue};
use rarian::db::dbm::{self, DBManager};
use rarian::RwTransaction;
use rarian::Transaction;

use crate::Settings;
use crate::segments::segments;

use futures::prelude::*;

pub fn add(log: &Logger, s: Settings, m: &clap::ArgMatches<'_>) {
    let target = m.value_of("target").expect("No value for `TARGET` set!");
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

    if m.is_present("batch") {
        add_batch(log, &mut txn, &mut db);
    } else {
        if let Some(i) = m.values_of("files") {
            let files: Vec<String> = i.map(str::to_string).collect();
            add_files(log, &mut txn, &mut db, files);
        } else {
            error!(log, "No files provided");
            return;
        }
    };

    if let Err(e) = Transaction::commit(txn) {
        error!(log, "Failed to commit transaction: {}", e);
    }
}

fn add_batch(log: &Logger, txn: &mut RwTransaction, db: &mut Database) {
    let stdin = io::stdin();
    let handle = stdin.lock();

    let s = stream::iter(handle.lines().filter_map(Result::ok)).map(|mut s| { s.push('\n'); s});
    match git_annex::add::add(s) {
        (f, Ok(s)) => {
            let f2 = s.for_each_concurrent(None, |r| match r {
                Ok((key, filename)) => {
                    match run_exiftool(filename) {
                        Ok(mut tag) => {
                            let mut format = HashMap::new();
                            if let Some(mimet) = tag.mime_type.take() {
                                format.insert(FormatKey::MimeType, mimet.into_boxed_str());
                            }
                            let ft = FileT { key, format };
                            let meta = tagtometa(tag);

                            let e = EntryT::new(ft, meta);

                            if let Err(e) = db.insert_rand(txn, &e) {
                                error!(log, "Could not add entry: {:?}", e);
                            }
                        }
                        Err(e) => {
                            error!(log, "Could not parse exiftool output: {}", e);
                        }
                    }
                    future::ready(())
                }
                Err(e) => {
                    error!(log, "Could not add a file: {}", e);
                    future::ready(())
                }
            });

            let f = f.map(|r| if let Err(e) = r { error!(log, "Failed to run git-annex: {}", e)});

            let g = futures::future::join(f, f2);
            futures::executor::block_on(g);
        }
        (_, Err(e)) => {
            error!(log, "Could not read git-annex stdout: {}", e);
        }
    };
}

fn add_files(log: &Logger, txn: &mut RwTransaction, db: &mut Database, files: Vec<String>) {
    let s = stream::iter(files.into_iter());
    match git_annex::add::add(s) {
        (f, Ok(s)) => {
            let f2 = s.for_each(|r| match r {
                Ok((key, filename)) => {
                    match run_exiftool(filename) {
                        Ok(mut tag) => {
                            let mut format = HashMap::new();
                            if let Some(mimet) = tag.mime_type.take() {
                                format.insert(FormatKey::MimeType, mimet.into_boxed_str());
                            }
                            let ft = FileT { key, format };
                            let meta = tagtometa(tag);

                            let e = EntryT::new(ft, meta);

                            if let Err(e) = db.insert_rand(txn, &e) {
                                error!(log, "Could not add entry: {:?}", e);
                            }
                        }
                        Err(e) => {
                            error!(log, "Could not parse exiftool output: {}", e);
                        }
                    }
                    future::ready(())
                }
                Err(e) => {
                    error!(log, "Could not add a file: {}", e);
                    future::ready(())
                }
            });


            let f = f.map(|r| if let Err(e) = r { error!(log, "Failed to run git-annex: {}", e)});
            let g = futures::future::join(f, f2);
            futures::executor::block_on(g);
        }
        (_, Err(e)) => {
            error!(log, "Could not read git-annex stdout: {}", e);
        }
    }
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

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum MaybeValueMaybeArray<V> {
    Value(V),
    Array(Vec<V>),
}

impl<V> MaybeValueMaybeArray<V> {
    fn into_iter(self) -> impl Iterator<Item=V> {
        use MaybeValueMaybeArray::*;
        match self {
            Value(s) => vec![s].into_iter(),
            Array(a) => a.into_iter(),
        }
    }
}

#[derive(Debug,Deserialize)]
struct Exiftag {
    #[serde(rename = "Title")]
    title: Option<MaybeValueMaybeArray<Box<str>>>,
    #[serde(rename = "Artist")]
    artist: Option<MaybeValueMaybeArray<Box<str>>>,
    #[serde(rename = "Comment")]
    comment: Option<MaybeValueMaybeArray<Box<str>>>,
    #[serde(rename = "Album")]
    album: Option<MaybeValueMaybeArray<Box<str>>>,
    #[serde(rename = "TrackNumber")]
    tracknr: Option<MaybeValueMaybeArray<i64>>,
    #[serde(rename = "Albumartist")]
    albumartist: Option<MaybeValueMaybeArray<Box<str>>>,

    #[serde(rename = "MIMEType")]
    mime_type: Option<String>,
}

fn tagtometa(tag: Exiftag) -> HashMap<Metakey, Metavalue> {
    let mut metadata = HashMap::new();
    if let Some(title) = tag.title {
        let title = title.into_iter().collect();
        metadata.insert(Metakey::Title, Metavalue::Title(title));
    }
    if let Some(artist) = tag.artist {
        let artist = artist.into_iter().collect();
        metadata.insert(Metakey::Artist, Metavalue::Artist(artist));
    }
    if let Some(comment) = tag.comment {
        let comment = comment.into_iter().collect();
        metadata.insert(Metakey::Comment, Metavalue::Comment(comment));
    }
    if let Some(album) = tag.album {
        let album = album.into_iter().collect();
        metadata.insert(Metakey::Album, Metavalue::Album(album));
    }
    if let Some(tracknr) = tag.tracknr {
        let tracknr = tracknr.into_iter().collect();
        metadata.insert(Metakey::TrackNumber, Metavalue::TrackNumber(tracknr));
    }
    if let Some(albumartist) = tag.albumartist {
        let albumartist = albumartist.into_iter().collect();
        metadata.insert(Metakey::Albumartist, Metavalue::Albumartist(albumartist));
    }

    metadata
}
