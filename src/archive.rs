use std::io;
use std::io::prelude::*;

use std::fs::File;
use std::path::PathBuf;

use clap::{App, ArgMatches};

use std::collections::HashMap;

use lmdb::Transaction;

use crate::decoders::Decoder;
use crate::Librarian;
use crate::git;
use rust_stemmers::{Algorithm, Stemmer};

use crate::database::{Key, Metadatabase, RwTransaction};
use crate::storage::Metadata;

pub const SUBCOMMAND: &str = "archive";

pub fn clap() -> App<'static, 'static> {
    clap_app!( @subcommand archive =>
        (about: "archive files")
        (@arg files: ... "List of files to operate on")
        // Alternatively, provide a filelist because a command line can only be so long.
        (@arg filelist: -f --filelist +takes_value conflicts_with[files] "File list, separated by newline. Use '-' for stdin")
    )
}

pub fn run(lib: Librarian, matches: &ArgMatches) {
    if let Some(filelist) = matches.value_of("filelist") {
        if filelist == "-" {
            let stdin = io::stdin();
            decode(lib, stdin.lock().lines().map(Result::unwrap))
        } else {
            match File::open(filelist) {
                Ok(f) => decode(lib, io::BufReader::new(f).lines().map(Result::unwrap)),
                Err(e) => error!("Failed to read filelist: {}", e),
            }
        }
    } else if let Some(files) = matches.values_of("files") {
        decode(lib, files.map(str::to_string))
    } else {
        println!("Provide either a filelist or a list of files, interactive mode is not yet implemented!");
    }
}

fn decode<I: Iterator<Item=String>>(lib: Librarian, iter: I) {
    let pb: Vec<std::path::PathBuf> = iter.map(PathBuf::from).collect();

    info!("Decoding files");
    let meta = Decoder::decode(&pb);
    info!("Annexing files");
    let keys = git::annex_add(&pb).unwrap();
    info!("Annexed files");

    let mut combined: Vec<(Key, Metadata)> = Vec::new();
    let mut keymap = HashMap::<&str, Key>::new();

    if !meta.is_empty() {

        let metaf = pb
            .iter()
            .map(|b| b.file_name().and_then(|os| os.to_str()))
            .zip(meta);
        info!("Storing Metadata");

        for (key, file) in keys.iter() {
            keymap.insert(&file, *key);
        }
        for i in metaf {
            match i {
                (Some(p), Ok(f)) => {
                    if let Some(k) = keymap.get(&p) {
                        combined.push((*k, f));
                        keymap.remove(&p);
                    } else {
                        warn!("file {} does not appear to have been annexed or was annexed twice", p);
                    }
                }
                (_, Err(e)) => {
                    error!("Failure to decode: {:?}", e);
                }
                _ => error!("Failure to decode path"),
            }
        }
    }

    if !keymap.is_empty() {
        warn!("Some files have been indexed but no metadata got extracted.");
        if log_enabled!(log::Level::Info) {
            info!("List of files:");
            for k in keymap.keys() {
                info!("    {}", k);
            }
        }
    }

    let db = Metadatabase::new(lib.dbm.create_named("main").unwrap());

    let en_stem = Stemmer::create(Algorithm::English);

    for (k,v) in combined.into_iter() {
        let r = lib.dbm.read().unwrap();
        let mut w = lib.dbm.write().unwrap();
        //index(dbi, &r, &mut w, &en_stem, k, &v);
        store(db, &mut w, &k, v);
        w.commit().unwrap();
    }

}


use std::collections::HashSet;

lazy_static! {
    static ref STOPWORDS: HashSet<&'static str> = {
        let words: &[&'static str] = &["a","able","about","across","after","all","almost","also","am","among","an","and","any","are","as","at","be","because","been","but","by","can","cannot","could","dear","did","do","does","either","else","ever","every","for","from","get","got","had","has","have","he","her","hers","him","his","how","however","i","if","in","into","is","it","its","just","least","let","like","likely","may","me","might","most","must","my","neither","no","nor","not","of","off","often","on","only","or","other","our","own","rather","said","say","says","she","should","since","so","some","than","that","the","their","them","then","there","these","they","this","tis","to","too","twas","us","wants","was","we","were","what","when","where","which","while","who","whom","why","will","with","would","yet","you","your"];
        let mut set = HashSet::new();
        for w in words.iter() {
            set.insert(*w);
        }

        set
    };
}

fn is_stopword(word: &str) -> bool {
    STOPWORDS.contains(word)
}

fn store(db: Metadatabase, w: &mut RwTransaction, key: &Key, val: Metadata) {
    db.put(w, key, val).unwrap();
}
