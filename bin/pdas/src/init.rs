use clap::{App, ArgMatches};
use git_annex::init::init;
use crate::{config, Librarian};
use crate::config::Remote;

pub const SUBCOMMAND: &'static str = "init";

pub fn clap() -> App<'static, 'static> {
    clap_app!( @subcommand setup =>
        (about: "Initialize the pdas repository")
    )
}

pub fn run(lib: Librarian, args: &ArgMatches) {
    let repo = config::repopath(&lib.config);
    let remotes: Vec<(String, String)> = lib.config.remotes.into_iter()
        .map(|(n, Remote { url })| { (n, url) })
        .collect();
    init(&repo, &remotes);
}
