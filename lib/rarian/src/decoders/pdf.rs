use std::collections::HashMap;
use std::path::PathBuf;
use crate::storage::{MetadataOwned, Metakey};
use crate::error::Result;
use std::process::Command;
use std::collections::VecDeque;

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
