pub mod text;
pub mod audio;

pub use text::*;
pub use audio::*;

use std::path::Path;
use std::path::PathBuf;

use crate::error::Error;

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use crate::Librarian;
use clap::{App, ArgMatches};

use log::Level::*;

pub use crate::storage::Storables;
use crate::storage::MetadataOwned;

pub const SUBCOMMAND: &str = "read";

pub fn clap() -> App<'static, 'static> {
    clap_app!(@subcommand read => 
        (about: "try to decode a file")
        (@arg file: *)
    )
}

pub fn run(lib: Librarian, matches: &ArgMatches) {
    if let Some(file) = matches.value_of("file") {
        let fpath = PathBuf::from(file);
        let vfp = vec![fpath];
        let result = Decoder::decode(&vfp);

        println!("{:?}", result.get(0));
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum FT {
    PDF,
    EPUB,
    FLAC,
    MPEG,
    Unrecognized,
}

pub struct Decoder;

impl Decoder {
    pub fn decode(files: &[PathBuf]) -> Vec<Result<MetadataOwned, Error>> {
        let mut out: Vec<Result<MetadataOwned, Error>> = Vec::with_capacity(files.len());
        let mut map: HashMap<FT, Vec<&Path>> = HashMap::new();

        for f in files {
            if let Some(mime) = tree_magic::from_filepath(f) {

                debug!("Inferred mime type of file {} as {}", f.display(), mime);

                let ft = match mime.as_str() {
                    "application/pdf" => FT::PDF,
                    "application/epub+zip" => FT::EPUB,
                    "audio/flac" => FT::FLAC,
                    "audio/mpeg" => FT::MPEG,
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
                    //FIXME I shouldn't need to clone here.
                    let pbi = v.iter().map(|p| Path::to_path_buf(p));
                    let mut r = PdfDecoder::new(pbi);
                    out.extend(&mut r);
                },
                FT::EPUB => {
                    //FIXME I shouldn't need to clone here.
                    let pbi = v.iter().map(|p| Path::to_path_buf(p));
                    let mut r = EpubDecoder::new(pbi);
                    out.extend(&mut r);
                },
                FT::FLAC => {
                    //FIXME I shouldn't need to clone here.
                    let pbi = v.iter().map(|p| Path::to_path_buf(p));
                    let mut r = FlacDecoder::new(pbi);
                    out.extend(&mut r);
                },
                FT::MPEG => {
                    //FIXME I shouldn't need to clone here.
                    let pbi = v.iter().map(|p| Path::to_path_buf(p));
                    let mut r = Id3Decoder::new(pbi);
                    out.extend(&mut r);
                },
                _ => {}
            }
        }

        out
    }
}

#[derive(Debug)]
pub enum DecodeError {
    Metaflac(metaflac::Error),
    Id3(id3::Error),
    Epub,
}
impl From<metaflac::Error> for DecodeError {
    fn from(e: metaflac::Error) -> Self {
        DecodeError::Metaflac(e)
    }
}
impl From<id3::Error> for DecodeError {
    fn from(e: id3::Error) -> Self {
        DecodeError::Id3(e)
    }
}
