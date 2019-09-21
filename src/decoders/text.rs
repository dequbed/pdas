use std::process::Command;

use std::path::Path;
use std::path::PathBuf;
use std::fs::File;
use std::collections::{HashMap, VecDeque};

use epub::doc::EpubDoc;

use crate::storage::{MetadataOwned, Metakey};
use crate::storage::Book;
use crate::error::Result;
use super::DecodeError;

pub struct PdfDecoder<I>{
    paths: I,
    vals: VecDeque<MetadataOwned>,
}
impl<I: Iterator<Item=PathBuf>> Iterator for PdfDecoder<I> {
    type Item = Result<MetadataOwned>;

    fn next(&mut self) -> Option<Self::Item> {
        // We try to batch calls to exiftool as far as possible
        if self.vals.is_empty() {
            // If there are no values stored we batch a few more
            match Command::new("exiftool")
                            .arg("-j")
                            .args(&mut self.paths)
                            .output() 
            {
                Ok(out) => {
                    if let Ok(s) = std::str::from_utf8(&out.stdout) {
                        if let Ok(mut ja) = json::parse(s) {
                            for j in ja.members_mut() {
                                let filename = j.remove("FileName").take_string().unwrap();
                                let author = j.remove("Author").take_string();
                                let title = j.remove("Title").take_string().unwrap_or_else(|| filename.clone());
                                let mut metamap = HashMap::new();

                                j.remove("Subject").take_string()
                                    .and_then(|v| metamap.insert(Metakey::Subject, v.into_boxed_str().into_boxed_bytes()));
                                j.remove("Description").take_string()
                                    .and_then(|v| metamap.insert(Metakey::Description, v.into_boxed_str().into_boxed_bytes()));
                                j.remove("CreateDate").take_string()
                                    .and_then(|v| metamap.insert(Metakey::Date, v.into_boxed_str().into_boxed_bytes()));
                                j.remove("DocumentID").take_string()
                                    .and_then(|v| metamap.insert(Metakey::Identifier, v.into_boxed_str().into_boxed_bytes()));

                                self.vals.push_back(MetadataOwned::new(title, author, filename, None, metamap));
                            }
                        }
                    } else {
                        error!("exiftool returned invalid UTF-8. Make sure your $LC_* variables are set to UTF-8!");
                    }
                }
                Err(e) => error!("failed to run exiftool: {}", e),
            }

            None
        } else {
            // There are still values stored from the last call to `exiftool`, return those first.
            self.vals.pop_front().map(Ok)
        }
    }
}
impl<I> PdfDecoder<I> {
    pub fn new(paths: I) -> Self {
        let vals = VecDeque::new();
        Self { paths, vals }
    }
}

pub struct EpubDecoder<I> {
    paths: I,
}
impl<I: Iterator<Item=PathBuf>> Iterator for EpubDecoder<I> {
    type Item = Result<MetadataOwned>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(p) = self.paths.next() {
            match EpubDoc::new(&p) {
                Ok(book) => {
                    let f = match File::open(&p) {
                        Ok(f) => f,
                        Err(e) => return Some(Err(e.into())),
                    };
                    let filesize = f.metadata().ok().map(|m| m.len() as usize);
                    let mut m = book.metadata;
                    let filename = p.file_name().map(|os| os.to_os_string().into_string().ok()).unwrap().unwrap();
                    let author = m.get_mut("creator").and_then(|v| v.pop());
                    let title = m.get_mut("title").and_then(|v| v.pop()).unwrap_or_else(|| filename.clone());


                    let mut metamap = HashMap::new();

                    m.get_mut("subject").and_then(|v| v.pop())
                        .and_then(|v| metamap.insert(Metakey::Subject, v.into_boxed_str().into_boxed_bytes()));
                    m.get_mut("description").and_then(|v| v.pop())
                        .and_then(|v| metamap.insert(Metakey::Description, v.into_boxed_str().into_boxed_bytes()));
                    m.get_mut("date").and_then(|v| v.pop())
                        .and_then(|v| metamap.insert(Metakey::Date, v.into_boxed_str().into_boxed_bytes()));
                    m.get_mut("identifier").and_then(|v| v.pop())
                        .and_then(|v| metamap.insert(Metakey::Identifier, v.into_boxed_str().into_boxed_bytes()));
                    m.get_mut("language").and_then(|v| v.pop())
                        .and_then(|v| metamap.insert(Metakey::Language, v.into_boxed_str().into_boxed_bytes()));
                    m.get_mut("publisher").and_then(|v| v.pop())
                        .and_then(|v| metamap.insert(Metakey::Publisher, v.into_boxed_str().into_boxed_bytes()));
                    m.get_mut("rights").and_then(|v| v.pop())
                        .and_then(|v| metamap.insert(Metakey::License, v.into_boxed_str().into_boxed_bytes()));

                    return Some(Ok(MetadataOwned::new(title, author, filename, filesize, metamap)));
                }
                Err(e) => {
                    return Some(Err(DecodeError::Epub.into()))
                }
            }
        }

        return None;
    }
}
impl<I> EpubDecoder<I> {
    pub fn new(paths: I) -> Self {
        Self { paths }
    }
}

// keys for creation time in my collection of epubs:
//
// - "Date" format "YYYY-MM-DD"
// - "Date" format "YYYY"
// - "Date" format "YYYY-MM-DDTHH:MM:SS+HH:MM"
// - "Date" format "2010-11-20T15:37:21.077000+00:00"
// - "Date" format "YYYY-MM"
