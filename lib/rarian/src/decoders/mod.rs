#[cfg(flac)]
mod flac;

#[cfg(id3)]
mod id3;

#[cfg(epub)]
mod epub;

#[cfg(pdf)]
mod pdf;

#[derive(Debug)]
pub enum DecodeError {
    #[cfg(flac)]
    Metaflac(metaflac::Error),

    #[cfg(id3)]
    Id3(id3::Error),

    #[cfg(epub)]
    Epub,
}

#[cfg(flac)]
impl From<metaflac::Error> for DecodeError {
    fn from(e: metaflac::Error) -> Self {
        DecodeError::Metaflac(e)
    }
}

#[cfg(id3)]
impl From<id3::Error> for DecodeError {
    fn from(e: id3::Error) -> Self {
        DecodeError::Id3(e)
    }
}
