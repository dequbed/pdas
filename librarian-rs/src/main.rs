#[macro_use]
extern crate clap;

#[macro_use]
extern crate log;
extern crate stderrlog;

extern crate tree_magic;

mod read;

fn main() {
    let matches = clap_app!(lib =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: "librarian is a tool to analyze and archive various types files more easily into a git-annex repository")
        (@arg CONFIG: -c --config +takes_value "Use the specified config file")
        (@arg verbose: -v --verbose ... "Be more verbose")
        (@arg quiet: -q --quiet "Be quiet")
        (subcommand: read::clap())
    ).get_matches();

    stderrlog::new()
        .module(module_path!())
        .quiet(matches.is_present("quiet"))
        .verbosity(matches.occurrences_of("verbose") as usize)
        .init()
        .unwrap();

    match matches.subcommand() {
        ("read", Some(args)) => read::decode(args),
        _ => println!("{}", matches.usage()),
    }
}
