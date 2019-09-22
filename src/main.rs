#[macro_use]
extern crate clap;

#[macro_use]
extern crate log;
extern crate stderrlog;

#[macro_use]
extern crate lazy_static;

extern crate bincode;
extern crate chrono;
extern crate directories;
extern crate libc;
extern crate lmdb;
extern crate rust_stemmers;
extern crate serde;
extern crate tree_magic;
extern crate toml;

use std::path::PathBuf;
use directories::ProjectDirs;
use std::fs::File;
use std::io::Read;

mod cli;
mod archive;
mod database;

mod decoders;

mod error;

mod config;

mod storage;

use cli::*;

fn main() {
    let matches = clap_app!(lib =>
        (@setting SubcommandRequiredElseHelp)
        (version: crate_version!())
        (author: crate_authors!())
        (about: "pdas is a tool to analyze and archive various types files more easily into a git-annex repository")
        (@arg CONFIG: -c --config +takes_value +global "Use the specified config file")
        (@arg verbose: -v --verbose +global ... "Be more verbose")
        (@arg quiet: -q --quiet +global "Be quiet")
        (subcommand: cli::archive::clap())
        (subcommand: db::clap())
        (subcommand: git::clap())
        (subcommand: decoders::clap())
        (subcommand: init::clap())
    ).get_matches();


    stderrlog::new()
        .module(module_path!())
        .quiet(matches.is_present("quiet"))
        .verbosity(matches.occurrences_of("verbose") as usize)
        .init()
        .unwrap();

    let librarian = Librarian::new(&matches);

    match matches.subcommand() {
        (cli::archive::SUBCOMMAND, Some(args)) => cli::archive::run(librarian, args),
        (decoders::SUBCOMMAND, Some(args)) => decoders::run(librarian, args),
        ("db", Some(args)) => db::run(librarian, args),
        (git::SUBCOMMAND, Some(args)) => git::run(librarian, args),
        (init::SUBCOMMAND, Some(args)) => init::run(librarian, args),
        _ => {}
    }
}

// Main application struct
pub struct Librarian {
    pub dbm: database::Manager,
}

impl Librarian {
    pub fn new(args: &clap::ArgMatches) -> Self {
        let mut dbmb = database::Manager::builder();
        dbmb.set_flags(lmdb::EnvironmentFlags::MAP_ASYNC | lmdb::EnvironmentFlags::WRITE_MAP);
        dbmb.set_map_size(10485760);
        dbmb.set_max_dbs(4);
        let dbm = database::Manager::from_builder(&config.database.basedir, dbmb).unwrap();

        Librarian {
            config,
            dbm,
        }
    }
}
