use clap::{App, ArgMatches};

use rarian::{Iter, DatabaseFlags};

use crate::Librarian;
use crate::error::Error;

use rarian::{
    Metadatabase,
    Stringindexdb,
    Metadata,
    Key,
    Occurance,
};

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
                        let r: Result<Occurance, Error> = Occurance::deserialize(vref).map_err(Error::Rarian);
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

            rarian::find(db, idb, r, needle);
        }
        _ => {}
    }
}
