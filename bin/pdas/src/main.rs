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
mod query;
use query::query;

fn main() {
    let m = clap_app!(pdas =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: crate_description!())
        (@arg CONFIG: -c --config +takes_value "Use a custom configuration file")
        (@arg VERBOSITY: -v --verbose ... "Be more verbose, specify multiple times")
        (@arg QUIET: -q --quiet conflicts_with("VERBOSITY") "Be less verbose")
        (@subcommand add =>
            (about: "Add a file to git-annex and the database")
            (@arg target: -t --target env("DB") +required "The target database")
            (@arg files: ... "Files to add")
            )
        (@subcommand query =>
            (about: "Query the database")
            (@arg target: -t --target env("TARGET") +required "The target database")
            (@arg query: "The query to run")
            )
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

    match m.subcommand() {
        ("add", Some(m)) => add(m),
        ("query", Some(m)) => query(m),
        (subcmd, _) => {
            crit!(log, "Unknown subcommand {}.", subcmd);
            exit(log, -2);
        },
    }
}

// Exit but flush the logger properly
fn exit(log: slog::Logger, code: i32) -> ! {
    std::mem::drop(log);
    std::process::exit(code)
}
