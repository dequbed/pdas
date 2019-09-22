use clap::{App, ArgMatches};

use lmdb::{Iter, IterDup, DatabaseFlags};

use crate::storage::Metadata;

use rust_stemmers::{Algorithm, Stemmer};

use crate::Librarian;
use crate::error::Error;

use crate::database::{Key, Metadatabase, Stringindexdb, Transaction, Occurance};

pub fn clap() -> App<'static, 'static> {
    clap_app!( @subcommand db =>
        (@setting SubcommandRequiredElseHelp)
        (about: "DB management subsystem")
        (@subcommand read =>
            (about: "read from the specified db")
            (@arg key: * "key to extract"))
        (@subcommand dump =>
            (about: "dump contents of the db"))
        (@subcommand index =>
            (about: "dump the index of the db"))
        (@subcommand search =>
            (about: "search for a word")
            (@arg term: * "search term"))
        (@subcommand create =>
            (about: "create all db files"))
    )
}

pub fn run(lib: Librarian, matches: &ArgMatches) {
    let db = Metadatabase::new(lib.dbm.create_named("main").unwrap());

    match matches.subcommand() {
        ("create", Some(args)) => {
            Stringindexdb::create(&lib.dbm, "title").unwrap();
        }
        ("read", Some(args)) => {
            let r = lib.dbm.read().unwrap();

            if let Some(k) = args.value_of("key") {
                if let Some(k) = Key::try_parse(k) {
                    match db.get(&r, &k) {
                        Ok(r) => println!("{:?}", r),
                        Err(e) => error!("Error: {:?}", e),
                    }
                } else {
                    error!("Invalid key");
                }
            } else {
                // Clap errors out before this gets hit due to setting SubcommandRequiredElseHelp
                unreachable!();
            }
        }
        ("dump", _) => {
            let r = lib.dbm.read().unwrap();

            let c = db.iter_start(&r).unwrap();
            if let Iter::Err(e) = c {
                error!("Iterator errored out, aborting: {:?}", e);
                return;
            }

            for i in c {
                match i {
                    Ok((kref, vref)) => {
                        let m = Metadata::decode(vref);
                        println!("{:?}: {:?}", kref, m);
                    },
                    Err(e) => {
                        error!("Retrieval errored out: {:?}", e);
                    }
                }
            }
        }
        ("index", _) => {
            let idb = Stringindexdb::open(&lib.dbm, "title").unwrap();
            let r = lib.dbm.read().unwrap();
            let is = idb.iter_start(&r).unwrap();

            if let Iter::Err(e) = is {
                error!("Iterator errored out, aborting: {:?}", e);
                return;
            }

            for i in is {
                match i {
                    Ok((kref, vref)) => {
                        let r: Result<Occurance, Error> = bincode::deserialize(vref).map_err(Error::Bincode);
                        match r {
                            Ok(o) => println!("{:?}: {:?}", kref, o),
                            Err(e) => error!("Failed to decode index value: {:?}", e),
                        }
                    },
                    Err(e) => {
                        error!("Retrieval errored out: {:?}", e);
                    }
                }
            }
        }
        ("search", Some(a)) => {
            let needle = a.value_of("term").unwrap();
            let idb = Stringindexdb::open(&lib.dbm, "title").unwrap();
            let r = lib.dbm.read().unwrap();

            find(db, idb, r, needle);
        }
        _ => {}
    }
}

fn find<T: Transaction>(db: Metadatabase, dbi: Stringindexdb, r: T, needle: &str) {
    let en_stem = Stemmer::create(Algorithm::English);
    let ndl = en_stem.stem(needle);
    let term: String = ndl.into();

    println!("Searching for {}", term);

    match dbi.get(&r, &term) {
        Ok(occ) => {
            println!("{:?}", occ);
        }
        Err(Error::LMDB(lmdb::Error::NotFound)) => {
            println!("No results");
        }
        Err(e) => {
            error!("while querying index db: {:?}", e);
        }
    }
}

// More sensible: What defines a Database in our context?
// 1. What Key-Type they use (MetaDB: SHA256E, TermDB: String)
// 2. What Value-Type they use (MetaDB: MetaValue, TermDB: TermOccurance)
// 3. Are they duplicate key types? (i.e. What kind of iterator do they use)
// 4. In general, what is their configuration like?
//
// A database uses bytestrings as Keys and Values. In Rust we can easily build that as DB<K:
// AsRef<[u8]>, V: AsRef<[u8]>>, i.e. generic over any type K and V that can both be dereferenced
// into bytestrings.
// A specific database (e.g. Metadata storage) is a composed struct that contains a version of that
// generic DB with both K and (maybe?) V bound to a specifc type. They should also define a custom
// wrapper around new() that enables them to configure the flags the DB is created with.
