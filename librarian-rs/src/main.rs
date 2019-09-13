#[macro_use]
extern crate clap;

#[macro_use]
extern crate log;
extern crate stderrlog;

extern crate tree_magic;

extern crate csv;
extern crate serde;

mod read;
mod db;
mod git;

fn main() {
    let matches = clap_app!(lib =>
        (@setting SubcommandRequiredElseHelp)
        (version: crate_version!())
        (author: crate_authors!())
        (about: "librarian is a tool to analyze and archive various types files more easily into a git-annex repository")
        (@arg CONFIG: -c --config +takes_value +global "Use the specified config file")
        (@arg verbose: -v --verbose +global ... "Be more verbose")
        (@arg quiet: -q --quiet +global "Be quiet")
        (subcommand: read::clap())
        (subcommand: db::clap())
        (subcommand: git::clap())
    ).get_matches();

    stderrlog::new()
        .module(module_path!())
        .quiet(matches.is_present("quiet"))
        .verbosity(matches.occurrences_of("verbose") as usize)
        .init()
        .unwrap();

    match matches.subcommand() {
        ("read", Some(args)) => read::decode(args),
        ("db", Some(args)) => db::run(args),
        ("git", Some(args)) => git::run(args),
        _ => {}
    }
}
