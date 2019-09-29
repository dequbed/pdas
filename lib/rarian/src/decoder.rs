use crate::error::{Result, Error};
use crate::storage::MetadataOwned;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use log::Level::*;
use futures::stream::Stream;
use futures::channel::mpsc;

use crate::decoders::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum FT {
    PDF,
    EPUB,
    FLAC,
    MPEG,
    Unrecognized,
}

// A ft-decoder will practically always do disk-io and may also do expensive stuff like calling to
// external programms, looking up information over network connections, etc. For that reason a
// decoder is a `futures::Stream`, may take arbitarily long to return an element and may reorder
// elements. The Decoder-Manager here should be prepared to handle all of those cases, e.g. by
// sending both the Key and Path to a decoder so that return-values don't need to be ordered in any
// way.


pub struct DecoderManager {
    /*
     *pdf: PdfDecoder,
     *epub: EpubDecoder,
     *flac: FlacDecoder,
     */
    mpeg_tx: mpsc::Sender<PathBuf>,
    mpeg: MpegDecoder<mpsc::Receiver<PathBuf>>,
}

impl DecoderManager {
    pub fn new() -> Self {
        let (mpeg_tx, mpeg_rx) = mpsc::channel(16);
        let mpeg = MpegDecoder::new(mpeg_rx);
        DecoderManager { mpeg_tx, mpeg }
    }
/*
 *        // TODO: Move this entire function over to using streams so we don't have to move an
 *        // iterator into n vectors but instead just split a stream into n new ones which can get
 *        // consumed by whatever.
 *        let mut out: Vec<Result<MetadataOwned>> = Vec::new();
 *        let mut map: HashMap<FT, Vec<PathBuf>> = HashMap::new();
 *
 *        for f in files {
 *            if let Some(mime) = tree_magic::from_filepath(&f) {
 *
 *                debug!("Inferred mime type of file {} as {}", f.display(), mime);
 *
 *                let ft = match mime.as_str() {
 *                    "application/pdf" => FT::PDF,
 *                    "application/epub+zip" => FT::EPUB,
 *                    "audio/flac" => FT::FLAC,
 *                    "audio/mpeg" => FT::MPEG,
 *                    _ => FT::Unrecognized
 *                };
 *
 *                if log_enabled!(Info) && ft == FT::Unrecognized {
 *                    info!("no decoder available for file {} with inferred mime type {}", f.display(), mime);
 *                }
 *
 *                match map.entry(ft) {
 *                    Entry::Occupied(mut e) => e.get_mut().push(f),
 *                    Entry::Vacant(e) => { e.insert(vec![f]); },
 *                }
 *            } else {
 *                warn!("No MIME for {}", f.display());
 *            }
 *        }
 *
 *        if let Some(u) = map.get(&FT::Unrecognized) {
 *            match u.len() {
 *                0 => {}
 *                1 => warn!("ignored 1 file because no decoder was available."),
 *                l => warn!("ignored {} files because no decoder was available.", l),
 *            }
 *        }
 *
 *        // FIXME: Optimize this code to *not* copy metadata thrice.
 *        //  One way would be to grab the underlying *mut ptr, construct a &mut slice from that
 *        //  (containing uninitialized values!) write into them, count how many and then as a last
 *        //  step reassemble the Vec. Needs unsafe but works.
 *
 *        for (k,v) in map.iter() {
 *            match *k {
 *                FT::PDF => {
 *                    //FIXME I shouldn't need to clone here.
 *                    let pbi = v.iter().map(|p| Path::to_path_buf(p));
 *                    let mut r = PdfDecoder::new(pbi);
 *                    out.extend(&mut r);
 *                },
 *                FT::EPUB => {
 *                    //FIXME I shouldn't need to clone here.
 *                    let pbi = v.iter().map(|p| Path::to_path_buf(p));
 *                    let mut r = EpubDecoder::new(pbi);
 *                    out.extend(&mut r);
 *                },
 *                FT::FLAC => {
 *                    //FIXME I MpegDecoderneed to clone here.
 *                    let pbi = v.iter().map(|p| Path::to_path_buf(p));
 *                    let mut r = FlacDecoder::new(pbi);
 *                    out.extend(&mut r);
 *                },
 *                FT::MPEG => {
 *                    //FIXME I shouldn't need to clone here.
 *                    let pbi = v.iter().map(|p| Path::to_path_buf(p));
 *                    let mut r = Id3Decoder::new(pbi);
 *                    out.extend(&mut r);
 *                },
 *                _ => {}
 *            }
 *        }
 *
 *        out
 *    }
 */
}
