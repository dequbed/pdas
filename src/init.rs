use std::path::Path;
use std::process::Command;

use clap::{App, ArgMatches};
use crate::Librarian;

pub const SUBCOMMAND: &'static str = "setup";

pub fn clap() -> App<'static, 'static> {
    clap_app!( @subcommand setup =>
        (@arg repo: *)
        (@arg dir: *)
    )
}

pub fn run(lib: Librarian, matches: &ArgMatches) {
    let dir = matches.value_of("dir").unwrap();
    let repo = matches.value_of("repo").unwrap();

    let status = Command::new("git")
        .arg("clone")
        .arg(repo)
        .arg(dir)
        .status();

    match status {
        Err(e) => {
            error!("Failed to start git clone: {}", e);
            return;
        },
        Ok(exit) => {
            if !exit.success() {
                if let Some(c) = exit.code() {
                    error!("git returned with error code: {}", c);
                } else {
                    error!("git was killed by a signal");
                }
                return;
            }
        }
    }

    // TODO give repos a description
    let status = Command::new("git")
        .arg("annex")
        .arg("init")
        .status();

    match status {
        Err(e) => {
            error!("Failed to init annex: {}", e);
            return;
        },
        Ok(exit) => {
            if !exit.success() {
                if let Some(c) = exit.code() {
                    error!("annex returned with error code: {}", c);
                } else {
                    error!("annex was killed by a signal");
                }
                return;
            }
        }
    }
}
