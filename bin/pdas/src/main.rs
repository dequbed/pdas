#![feature(async_closure)]

#[macro_use]
extern crate clap;
#[macro_use]
extern crate slog;

use slog::Level;
use slog::Drain;

mod settings;
use settings::Settings;
mod add;
use add::add;
use add::index;
mod query;
use query::query;
mod create;
use create::create;
mod dump;
use dump::dump;
mod import;
use import::import;
mod export;
use export::export;

mod segments;

use futures::executor::block_on;

fn main() {
    let m = clap_app!(pdas =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: crate_description!())
        (@arg CONFIG: -c --config +takes_value "Use a custom configuration file")
        (@arg VERBOSITY: -v --verbose ... "Be more verbose, specify multiple times")
        (@arg QUIET: -q --quiet conflicts_with("VERBOSITY") "Be less veterbose")
        (@subcommand add =>
            (about: "Add a file to git-annex and the database")
            (@arg target: -t --target env("TARGET") +required "The target database")
            (@arg files: ... "Files to add")
            (@arg batch: --batch -b conflicts_with("files") "Batch mode; expect files on stdin, separated by newlines"))
        (@subcommand index =>
            (about: "Add a file the database without adding to git-annex")
            (@arg target: -t --target env("TARGET") +required "The target database")
            (@arg files: ... +required "Files to add"))
        (@subcommand query =>
            (about: "Query the database")
            (@arg target: -t --target env("TARGET") +required "The target database")
            (@arg query: ... "The query to run"))
        (@subcommand create =>
            (about: "Create a database with a schema")
            (@arg target: -t --target env("TARGET") +required "The target database")
            (@arg schema: -s --schema +required +takes_value "The schema file"))
        (@subcommand dump => 
            (about: "Dump the database")
            (@arg target: -t --target env("TARGET") +required "The target database"))
        (@subcommand import =>
            (about: "Import a directory of entries into the database")
            (@arg target: -t --target env("TARGET") +required "The target database")
            (@arg entries: -d --directory +required +takes_value "Directory of entries"))
        (@subcommand export =>
            (about: "Export the database into a directory")
            (@arg target: -t --target env("TARGET") +required "The target database")
            (@arg entries: -d --directory +required +takes_value "Directory to export to"))
    ).get_matches();

    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let log = slog::Logger::root(drain, o!());

    let mut s = match Settings::new(&log, m.value_of("CONFIG")) {
        Ok(s) => s,
        Err(e) => {
            crit!(log, "Could not read configuration: {}", e);
            exit(log, -1);
        }
    };

    let vs = m.occurrences_of("VERBOSITY");
    if vs == 1 {
        s.set_loglevel(Level::Warning);
    } else if vs >= 2 {
        s.set_loglevel(Level::Info);
    }

    debug!(log, "Settings: {:?}", s);

    match m.subcommand() {
        ("add", Some(m)) => {
            add(&log, s, m);
            exit(log, 0);
        },
        ("index", Some(m)) => {
            index(&log, s, m);
            exit(log, 0);
        },
        ("query", Some(m)) => {
            let f = query(&log, s, m);
            block_on(f);
            exit(log, 0);
        },
        ("create", Some(m)) => {
            let f = create(&log, s, m);
            block_on(f);
            exit(log, 0);
        },
        ("dump", Some(m)) => {
            let f = dump(&log, s, m);
            block_on(f);
            exit(log, 0);
        },
        ("import", Some(m)) => {
            let f = import(&log, s, m);
            block_on(f);
            exit(log, 0);
        },
        ("export", Some(m)) => {
            let f = export(&log, s, m);
            block_on(f);
            exit(log, 0);
        },
        (subcmd, _) => {
            crit!(log, "Unknown subcommand {}.", subcmd);
            exit(log, -2);
        },
    }
}

// std::process::exit but flush the logger properly
fn exit(log: slog::Logger, code: i32) -> ! {
    std::mem::drop(log);
    std::process::exit(code)
}
