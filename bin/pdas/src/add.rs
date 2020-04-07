use std::path::Path;
use std::io::Read;
use std::io;
use std::fs;
use std::collections::{HashMap, HashSet};

use clap;

use serde::Deserialize;


use rarian::schema::Schema;
use rarian::query::Transaction;
use rarian::db::dbm::{self, DBManager};
use rarian::db::{Database, UUID};
use rarian::db::meta::Metakey;
use rarian::db::entry::{EntryOwn, EntryT};
use rarian::query::Querier;

pub fn add(m: &clap::ArgMatches) {

    let dbdir = Path::new("/tmp/asdf");

    let mut dbmb = DBManager::builder();
    dbmb.set_flags(dbm::EnvironmentFlags::MAP_ASYNC | dbm::EnvironmentFlags::WRITE_MAP);
    dbmb.set_max_dbs(126);
    dbmb.set_map_size(10485760);
    let dbm = DBManager::from_builder(dbdir.as_ref(), dbmb).unwrap();

    //let mut file = fs::File::open("/tmp/asdf/music.yml").unwrap();
    //let mut buf = Vec::new();
    //file.read_to_end(&mut buf).unwrap();
    //let schema = Schema::from_yaml(&buf[..]).unwrap();

    let mut txn = dbm.write().unwrap();
    //Database::create(&mut wtxn, "music", schema).unwrap();
    let mut db = Database::open(&txn, "music").unwrap();

    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut buffer = String::new();
    handle.read_to_string(&mut buffer).unwrap();

    // let tags: Vec<Exiftag> = serde_json::from_str(&buffer).unwrap();
    // for tag in tags.into_iter() {
    //     let e = tagtoentry(tag);
    //     let uuid = UUID::generate();
    //     db.insert(&mut txn, &uuid, &e).unwrap();
    // }

    let qr = Querier::new(&txn, &db);
    //let q = parse("");

    Transaction::commit(txn).unwrap();

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

    EntryT::newv(HashSet::new(), metadata)
}
