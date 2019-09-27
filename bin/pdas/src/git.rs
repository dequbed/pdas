use clap::{App, ArgMatches};

use std::process::{Command, Stdio};
use std::io::{self, BufReader, BufRead, Write};

pub const SUBCOMMAND: &'static str = "git";

use crate::Librarian;

pub fn clap() -> App<'static, 'static> {
    clap_app!( @subcommand git =>
        (@setting TrailingVarArg)
        (about: "direct git subsystem")
        (@arg cmd: ... "command to forward to git")
    )
}

pub fn run(lib: Librarian, args: &ArgMatches) {
    let mut git = Command::new("git");

    if let Some(vargs) = args.values_of_os("cmd") {
        git.args(vargs);
    }

    match git.spawn() {
        Ok(mut child) => { child.wait().expect("git process failed to start"); },
        Err(e) => { error!("Failed to start `git`: {}", e) },
    }
}
