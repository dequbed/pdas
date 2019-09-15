#[macro_use]
extern crate clap;

#[macro_use]
extern crate log;
extern crate stderrlog;

extern crate tree_magic;

extern crate csv;
extern crate serde;

extern crate lmdb;
extern crate directories;

extern crate libc;

extern crate bincode;

extern crate chrono;

use std::path::Path;

mod archive;
mod db;
mod git;

mod decoders;

mod error;

fn main() {
    let matches = clap_app!(lib =>
        (@setting SubcommandRequiredElseHelp)
        (version: crate_version!())
        (author: crate_authors!())
        (about: "librarian is a tool to analyze and archive various types files more easily into a git-annex repository")
        (@arg CONFIG: -c --config +takes_value +global "Use the specified config file")
        (@arg verbose: -v --verbose +global ... "Be more verbose")
        (@arg quiet: -q --quiet +global "Be quiet")
        (@arg dbdir: --dbdir +takes_value "Database directory")
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

    let librarian = Librarian::new(matches.value_of("dbdir").map(Path::new));

    match matches.subcommand() {
        (archive::SUBCOMMAND, Some(args)) => archive::run(librarian, args),
        (decoders::SUBCOMMAND, Some(args)) => decoders::run(librarian, args),
        ("db", Some(args)) => db::run(librarian, args),
        ("git", Some(args)) => git::run(args),
        _ => {}
    }
}

// Main application struct
pub struct Librarian {
    pub dbm: db::Manager,
}

use directories::ProjectDirs;

impl Librarian {
    pub fn new(dir: Option<&Path>) -> Self {
        let path;
        let proj_dir;
        if let Some(p) = dir {
            path = p;
        } else {
            proj_dir = ProjectDirs::from("org", "Paranoidlabs", "Librarian").unwrap();
            path = proj_dir.data_dir();
        }
        let mut dbmb = db::Manager::builder();
        dbmb.set_flags(db::EnvironmentFlags::MAP_ASYNC | db::EnvironmentFlags::WRITE_MAP);
        let dbm = db::Manager::from_builder(path, dbmb).unwrap();

        Librarian {
            dbm
        }
    }
}
