#[macro_use]
extern crate clap;

#[macro_use]
extern crate log;
extern crate stderrlog;

extern crate directories;
extern crate toml;

#[cfg(test)]
#[macro_use]
extern crate maplit;

extern crate rarian;

mod config;
mod error;

use std::path::Path;

fn main() {
    let matches = clap_app!(lib =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: "pdas is a tool to analyze and archive various types files more easily into a git-annex repository")
        (@arg CONFIG: -c --config +takes_value +global "Use the specified config file")
        (@arg verbose: -v --verbose +global ... "Be more verbose")
        (@arg quiet: -q --quiet +global "Be quiet")
    ).get_matches();


    stderrlog::new()
        .module(module_path!())
        .quiet(matches.is_present("quiet"))
        .verbosity(matches.occurrences_of("verbose") as usize)
        .init()
        .unwrap();

    let librarian = Librarian::new(&matches);

    match matches.subcommand() {
        _ => {}
    }
}

 // Main application struct
pub struct Librarian {
    pub dbm: rarian::Manager,
    pub config: config::Config,
}

impl Librarian {
    pub fn new(args: &clap::ArgMatches) -> Self {
        let config = match config::read_or_create(args.value_of("CONFIG")) {
            Ok(c) => c,
            Err(e) => {
                error!("failed to read config: {:?}", e);
                std::process::exit(-1);
            }
        };

        let dbdir = config::dbpath(&config);
        if !Path::exists(&dbdir) {
            match std::fs::create_dir_all(&dbdir) {
                Ok(_) => {},
                Err(e) => {
                    error!("failed to create database directory {}: {:?}", dbdir.display(), e);
                    std::process::exit(-1);
                }
            }
        } else if Path::is_file(&config.path) {
            error!("database directory {} is a file", config.path.display());
            std::process::exit(-1);
        }

        let mut dbmb = rarian::Manager::builder();
        dbmb.set_flags(rarian::EnvironmentFlags::MAP_ASYNC | rarian::EnvironmentFlags::WRITE_MAP);
        dbmb.set_map_size(10485760);
        dbmb.set_max_dbs(4);
        let dbm = rarian::Manager::from_builder(&dbdir, dbmb).unwrap();


        Librarian {
            dbm,
            config,
        }
    }
}
