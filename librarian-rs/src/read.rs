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

use csv::Writer;

use serde::Serialize;

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

    let mut out: Vec<Book> = Vec::new();

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
            "application/epub+zip" => { let _: Option<()> = read_epub(&path).and_then(|v| {out.push(v); None}); },
            _ => {}
        }
    }

    // PDF benefits from batch reading since we pass it to a different tool.
    if let Some(pdfs) = map.get(&Decoders::PDF) {
        let mut po = read_pdf_batch(pdfs);
        out.append(&mut po);
    }

    let mut w = Writer::from_path("/tmp/out.csv").unwrap();
    for o in out {
        w.serialize(o).unwrap();
    }
    w.flush().unwrap()
}

#[derive(Debug, PartialEq, Eq, Serialize)]
struct Book {
    filename: Option<String>,
    author: Option<String>,
    title: Option<String>,
    subject: Option<String>,
    description: Option<String>,
    date: Option<String>,
    identifier: Option<String>,
    language: Option<String>,
    publisher: Option<String>,
    license: Option<String>,
}

fn read_pdf_batch(paths: &Vec<String>) -> Vec<Book> {
    let mut rv = Vec::new();
    match Command::new("exiftool")
                    .arg("-j")
                    .args(paths.iter())
                    .output() 
    {
        Ok(out) => {
            if let Ok(s) = std::str::from_utf8(&out.stdout) {
                if let Ok(mut ja) = json::parse(s) {
                    for j in ja.members_mut() {
                        let b = Book {
                            filename: j.remove("FileName").take_string(),
                            author: j.remove("Author").take_string(),
                            title: j.remove("Title").take_string(),
                            subject: j.remove("Subject").take_string(),
                            description: j.remove("Description").take_string(),
                            date: j.remove("CreateDate").take_string(),
                            identifier: j.remove("DocumentID").take_string(),
                            language: None,
                            publisher: None,
                            license: None,
                        };
                        rv.push(b);
                    }
                }
            } else {
                error!("exiftool returned invalid UTF-8. Make sure your $LC_* variables are set to UTF-8!");
            }
        }
        Err(e) => error!("Failed to run exiftool: {}", e),
    }

    return rv;
}

fn read_epub(path: &Path) -> Option<Book> {
    match EpubDoc::new(path) {
        Ok(book) => {
            let mut m = book.metadata;
            return Some(Book {
                filename: path.file_name().and_then(|os| os.to_os_string().into_string().ok()),
                author: m.get_mut("creator").and_then(|v| v.pop()),
                title: m.get_mut("title").and_then(|v| v.pop()),
                subject: m.get_mut("subject").and_then(|v| v.pop()),
                description: m.get_mut("description").and_then(|v| v.pop()),
                date: m.get_mut("date").and_then(|v| v.pop()),
                identifier: m.get_mut("identifier").and_then(|v| v.pop()),
                language: m.get_mut("language").and_then(|v| v.pop()),
                publisher: m.get_mut("publisher").and_then(|v| v.pop()),
                license: m.get_mut("rights").and_then(|v| v.pop()),
            });
        }
        Err(e) => error!("Failed to read EPUB {}: {}", path.display(), e),
    }

    None
}
