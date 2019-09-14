use std::io;
use std::io::prelude::*;

use std::fs::File;
use std::path::PathBuf;

use clap::{App, ArgMatches};

use std::collections::HashMap;

use crate::decoders::Decoder;
use crate::decoders::Storables;
use crate::Librarian;
use crate::git;

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

    let mut keymap = HashMap::<&str, &str>::new();
    if !meta.is_empty() {
        info!("Storing Metadata");

        let db = lib.dbm.open().unwrap();
        let mut w = lib.dbm.write().unwrap();

        for (key, file) in keys.iter() {
            keymap.insert(&file, &key);
        }
        for f in meta.iter() {
            match f {
                Ok(s) => {
                    if let Some(k) = keymap.get(s.filename()) {
                        db.put_store(&mut w, k.as_bytes(), s).unwrap();
                        keymap.remove(s.filename());
                    } else {
                        warn!("file {} does not appear to have been annexed or was annexed twice", s.filename());
                    }
                }
                Err(e) => {
                    error!("Failure to decode: {:?}", e);
                }
            }
        }

        w.commit().unwrap();
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
}

// Archiving a file:
// 1. Read, parse metadata, go for it.
// 2. annex, get the key.
// 3. Put the metadata in the table
