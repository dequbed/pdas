use std::io;
use std::io::prelude::*;

use std::fs::File;

use clap::{App, ArgMatches};

use crate::Librarian;

pub const SUBCOMMAND: &str = "archive";

pub fn clap() -> App<'static, 'static> {
    clap_app!( @subcommand archive =>
        (about: "archive files")
        (@arg files: ... "List of files to operate on")
        // Alternatively, provide a filelist because a command line can only be so long.
        (@arg filelist: -f --filelist +takes_value conflicts_with[files] "File list, separated by newline. Use '-' for stdin")
    )
}

pub fn run(lib: Librarian, matches: &ArgMatches) {
    if let Some(filelist) = matches.value_of("filelist") {
        if filelist == "-" {
            let stdin = io::stdin();
            /*decode(lib, stdin.lock().lines().map(Result::unwrap))*/
        } else {
            match File::open(filelist) {
                Ok(f) => {}
                    //decode(lib, io::BufReader::new(f).lines().map(Result::unwrap)),
                Err(e) => error!("Failed to read filelist: {}", e),
            }
        }
    } else if let Some(files) = matches.values_of("files") {
        //decode(lib, files.map(str::to_string))
    } else {
        error!("Provide either a filelist or a list of files, interactive mode is not yet implemented!");
    }
}
