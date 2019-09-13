use std::io;
use std::io::prelude::*;

use std::fs::File;
use std::path::Path;

use clap::{App, ArgMatches};

use tree_magic;

use epub::doc::EpubDoc;

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use std::process::Command;

pub fn clap() -> App<'static, 'static> {
    clap_app!( @subcommand read =>
        (about: "read a list of files and try to extract their metadata")
        (@arg files: ... "List of files to operate on")
        // Alternatively, provide a filelist because a command line can only be so long. (CAN BE
        // STDIN!)
        (@arg filelist: -f --filelist +takes_value conflicts_with[files] "Filelist, separated by newline. Use '-' for stdin")
    )
}

pub fn decode(matches: &ArgMatches) {
    if let Some(filelist) = matches.value_of("filelist") {
        if filelist == "-" {
            let stdin = io::stdin();
            run(stdin.lock().lines().map(Result::unwrap))
        } else {
            match File::open(filelist) {
                Ok(f) => run(io::BufReader::new(f).lines().map(Result::unwrap)),
                Err(e) => error!("Failed to read filelist: {}", e),
            }
        }
    } else if let Some(files) = matches.values_of("files") {
        run(files.map(str::to_string))
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum Decoders {
    PDF,
}

fn run<I: Iterator<Item=String>>(iter: I) {
    let mut map: HashMap<Decoders, Vec<String>> = HashMap::new();

    for f in iter {
        let path = Path::new(&f);
        let mtype = tree_magic::from_filepath(path);

        info!("Infered type of file {} as {}", f, mtype);

        match mtype.as_str() {
            "application/pdf" => {
                match map.entry(Decoders::PDF) {
                    Entry::Occupied(mut e) => e.get_mut().push(f),
                    Entry::Vacant(e) => { e.insert(vec![f]); },
                }
            }
            "application/epub+zip" => read_epub(&path),
            _ => {}
        }
    }

    // PDF benefits from batch reading since we pass it to a different tool.
    read_pdf_batch(&map[&Decoders::PDF]);
}

fn read_pdf_batch(paths: &Vec<String>) {
    match Command::new("exiftool")
                    .arg("-j")
                    .args(paths.iter())
                    .output() 
    {
        Ok(out) => {
            if let Ok(s) = std::str::from_utf8(&out.stdout) {
                let j = json::parse(s);
                println!("{:?}", j);
            } else {
                error!("exiftool returned invalid UTF-8. Make sure your $LC_* variables are set to UTF-8!");
            }
        }
        Err(e) => error!("Failed to run exiftool: {}", e),
    }

}

fn read_epub(path: &Path) {
    match EpubDoc::new(path) {
        Ok(book) => println!("{:?}", book.metadata),
        Err(e) => error!("Failed to read EPUB {}: {}", path.display(), e),
    }
}
