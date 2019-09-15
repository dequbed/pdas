pub mod text;
pub mod audio;

pub use text::*;
pub use audio::*;

use serde::{Serialize, Deserialize};

use std::path::Path;
use std::path::PathBuf;

use crate::error::Error;

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use log::Level::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Storables {
    Text(Book),
    Audio(Song),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum FT {
    PDF,
    EPUB,
    FLAC,
    Unrecognized,
}

pub struct Decoder;

impl Decoder {
    pub fn decode(files: &[PathBuf]) -> Vec<Result<Storables, Error>> {
        let mut out: Vec<Result<Storables, Error>> = Vec::with_capacity(files.len());
        let mut map: HashMap<FT, Vec<&Path>> = HashMap::new();

        for f in files {
            if let Some(mime) = tree_magic::from_filepath(f) {

                debug!("Inferred mime type of file {} as {}", f.display(), mime);

                let ft = match mime.as_str() {
                    "application/pdf" => FT::PDF,
                    "application/epub+zip" => FT::EPUB,
                    "audio/flac" => FT::FLAC,
                    _ => FT::Unrecognized
                };

                if log_enabled!(Info) && ft == FT::Unrecognized {
                    info!("no decoder available for file {} with inferred mime type {}", f.display(), mime);
                }

                match map.entry(ft) {
                    Entry::Occupied(mut e) => e.get_mut().push(f),
                    Entry::Vacant(e) => { e.insert(vec![f]); },
                }
            } else {
                warn!("No MIME for {}", f.display());
            }
        }

        if let Some(u) = map.get(&FT::Unrecognized) {
            match u.len() {
                0 => {}
                1 => warn!("ignored 1 file because no decoder was available."),
                l => warn!("ignored {} files because no decoder was available.", l),
            }
        }

        // FIXME: Optimize this code to *not* copy metadata thrice.
        //  One way would be to grab the underlying *mut ptr, construct a &mut slice from that
        //  (containing uninitialized values!) write into them, count how many and then as a last
        //  step reassemble the Vec. Needs unsafe but works.

        for (k,v) in map.iter() {
            match *k {
                FT::PDF => {
                    let mut r = PdfDecoder::decode(v)
                        .into_iter()
                        .map(Storables::Text)
                        .map(Ok)
                        .collect();
                    out.append(&mut r);
                },
                FT::EPUB => {
                    let mut r = EpubDecoder::decode(v)
                        .into_iter()
                        .map(Storables::Text)
                        .map(Ok)
                        .collect();
                    out.append(&mut r);
                },
                FT::FLAC => {
                    let mut r = FlacDecoder::decode(v)
                        .into_iter()
                        .map(Storables::Audio)
                        .map(Ok)
                        .collect();
                    out.append(&mut r);
                }
                _ => {}
            }
        }

        out
    }
}
