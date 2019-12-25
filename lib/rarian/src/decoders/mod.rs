mod flac;
pub use flac::FlacDecoder;

mod mpeg;
pub use mpeg::MpegDecoder;

mod epub;
pub use self::epub::EpubDecoder;

mod pdf;
pub use pdf::PdfDecoder;

#[derive(Debug)]
pub enum DecodeError {
    Metaflac(metaflac::Error),
    Id3(::id3::Error),
    Epub,
    NotFound,
}
