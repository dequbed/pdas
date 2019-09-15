#[macro_use]
extern crate clap;

#[macro_use]
extern crate log;
extern crate stderrlog;

#[macro_use]
extern crate lazy_static;

extern crate tree_magic;

extern crate csv;
extern crate serde;

extern crate lmdb;
extern crate directories;

extern crate libc;

extern crate bincode;

extern crate chrono;

extern crate toml;

extern crate rust_stemmers;

use std::path::PathBuf;
use directories::ProjectDirs;
use std::fs::File;
use std::io::Read;

mod archive;
mod db;
mod git;

mod decoders;

mod error;

mod config;

fn main() {
    let matches = clap_app!(lib =>
        (@setting SubcommandRequiredElseHelp)
        (version: crate_version!())
        (author: crate_authors!())
        (about: "librarian is a tool to analyze and archive various types files more easily into a git-annex repository")
        (@arg CONFIG: -c --config +takes_value +global "Use the specified config file")
        (@arg verbose: -v --verbose +global ... "Be more verbose")
        (@arg quiet: -q --quiet +global "Be quiet")
        (subcommand: archive::clap())
        (subcommand: db::clap())
        (subcommand: git::clap())
        (subcommand: decoders::clap())
    ).get_matches();


    stderrlog::new()
        .module(module_path!())
        .quiet(matches.is_present("quiet"))
        .verbosity(matches.occurrences_of("verbose") as usize)
        .init()
        .unwrap();

    let librarian = Librarian::new(&matches);

    match matches.subcommand() {
        (archive::SUBCOMMAND, Some(args)) => archive::run(librarian, args),
        (decoders::SUBCOMMAND, Some(args)) => decoders::run(librarian, args),
        ("db", Some(args)) => db::run(librarian, args),
        (git::SUBCOMMAND, Some(args)) => git::run(librarian, args),
        _ => {}
    }
}

// Main application struct
pub struct Librarian {
    pub config: config::Config,
    pub dbm: db::Manager,
}

impl Librarian {
    pub fn new(args: &clap::ArgMatches) -> Self {
        let proj_dir = ProjectDirs::from("org", "paranoidlabs", "librarian").unwrap();
        let configp = args.value_of("CONFIG")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                proj_dir.config_dir().with_file_name("librarian.toml")
            });

        let mut config = config::Config::default();
        match File::open(configp) {
            Ok(mut f) => {
                let mut buf = String::new();
                if let Err(e) = f.read_to_string(&mut buf) {
                    error!("Failed to read config file: {}", e);
                } else {
                    let cr = config::Config::from_str(&buf);
                    match cr {
                        Ok(c) => { config = c },
                        Err(e) => error!("Failed to read config file: {}", e),
                    }
                }
            }
            Err(e) => error!("Failed to open config file: {}", e),
        }

        let mut dbmb = db::Manager::builder();
        dbmb.set_flags(db::EnvironmentFlags::MAP_ASYNC | db::EnvironmentFlags::WRITE_MAP);
        dbmb.set_map_size(10485760);
        dbmb.set_max_dbs(4);
        let dbm = db::Manager::from_builder(&config.database.basedir, dbmb).unwrap();

        Librarian {
            config,
            dbm,
        }
    }
}
