use std::process::Command;

use std::path::Path;

use serde::{Serialize, Deserialize};

use epub::doc::EpubDoc;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Book {
    pub filename: String,
    pub author: Option<String>,
    pub title: Option<String>,
    pub subject: Option<String>,
    pub description: Option<String>,
    pub date: Option<String>,
    pub identifier: Option<String>,
    pub language: Option<String>,
    pub publisher: Option<String>,
    pub license: Option<String>,
}


pub struct PdfDecoder;

impl PdfDecoder {
    pub fn decode(paths: &[&Path]) -> Vec<Book> {
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
                                filename: j.remove("FileName").take_string().unwrap(),
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
            Err(e) => error!("failed to run exiftool: {}", e),
        }

        return rv;
    }
}

pub struct EpubDecoder;

impl EpubDecoder {
    pub fn decode(paths: &[&Path]) -> Vec<Book> {
        let mut v = Vec::with_capacity(paths.len());

        for path in paths {
            if let Some(b) = Self::decode_single(path) {
                v.push(b);
            }
        }

        v
    }
    pub fn decode_single(path: &Path) -> Option<Book> {
        match EpubDoc::new(path) {
            Ok(book) => {
                let mut m = book.metadata;
                return Some(Book {
                    filename: path.file_name().map(|os| os.to_os_string().into_string().ok()).unwrap().unwrap(),
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
}
