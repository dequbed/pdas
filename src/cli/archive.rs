use std::io;
use std::io::prelude::*;

use std::fs::File;

use clap::{App, ArgMatches};

use crate::Librarian;

use crate::archive::decode;

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
            decode(lib, stdin.lock().lines().map(Result::unwrap))
        } else {
            match File::open(filelist) {
                Ok(f) => decode(lib, io::BufReader::new(f).lines().map(Result::unwrap)),
                Err(e) => error!("Failed to read filelist: {}", e),
            }
        }
    } else if let Some(files) = matches.values_of("files") {
        decode(lib, files.map(str::to_string))
    } else {
        println!("Provide either a filelist or a list of files, interactive mode is not yet implemented!");
    }
}

// Import process
// 1. Check if the file is already annexed, if so => SKIP
// 1.1 git-annex calckey
// 2. Else read metadata, annex the file into the object store
// 3. Calculate the path, move the link
// 4. Do 1-3 for all files, then commit & sync
//
// Output of git-annex import:
// When skipping because a file was already annexed:
// {"command":"import","note":"duplicate; skipping","success":true,"file":"USB Type-C Specification Release 1.1.pdf"}
//
// When importing a file:
// {"command":"import","success":true,"key":"SHA256E-s562633--bf8560a3fe8ff8e87dffa1b0d1caf868fb64862018d2377998ad087dd1631a13.pdf","file":"USB-C_Source_Power_Test_Specification_2018_06_01.pdf"}
//
// When failing because called with --duplicate but file was already annexed:
// {"command":"import","success":false,"file":"USB-C_Source_Power_Test_Specification_2018_06_01.pdf"}
//
// When importing with --skip-duplicates defined:
// {"command":"import","success":true,"key":"SHA256E-s8692742--70e9ef5ae0e8d53933740f5a67326db0178e10f196b324e5ba1cf49956ebb5eb.1.pdf","file":"USB Type-C Specification Release 1.1.pdf"}
