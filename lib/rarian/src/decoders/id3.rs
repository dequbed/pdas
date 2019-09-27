pub struct Id3Decoder<I> {
    paths: I
}
impl<I: Iterator<Item=PathBuf>> Iterator for Id3Decoder<I> {
    type Item = Result<MetadataOwned>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(p) = self.paths.next() {
            let f = File::open(&p).unwrap();
            match Id3Tag::read_from(&f) {
                Ok(tag) => {
                    let filename = p.file_name().and_then(OsStr::to_str).map(str::to_string).unwrap();
                    let title = tag.title().unwrap_or_else(|| &filename).to_string();
                    let author = tag.artist().map(|s| s.to_string());
                    let filesize = f.metadata().ok().map(|m| m.len() as usize);

                    let mut metamap = HashMap::new();

                    if let Some(album) = tag.album() { 
                        let albuf = album.to_string().into_boxed_str().into_boxed_bytes();
                        metamap.insert(Metakey::Album, albuf);
                    }
                    if let Some(genre) = tag.genre() {
                        let genbuf = genre.to_string().into_boxed_str().into_boxed_bytes();
                        metamap.insert(Metakey::Genre, genbuf);
                    }
                    if let Some(track) = tag.track() {
                        let buf = Box::new(track.to_le_bytes());
                        metamap.insert(Metakey::Track, buf);
                    }
                    if let Some(ttrack) = tag.total_tracks() {
                        let buf = Box::new(ttrack.to_le_bytes());
                        metamap.insert(Metakey::Totaltracks, buf);
                    }
                    if let Some(artist) = tag.album_artist() { 
                        let albuf = artist.to_string().into_boxed_str().into_boxed_bytes();
                        metamap.insert(Metakey::Albumartist, albuf);
                    }

                    let m = MetadataOwned::new(title, author, filename, filesize, metamap);
                    return Some(Ok(m));
                }
                Err(e) => {
                    error!("Failed to read FLAC tag: {}", e);
                    let e: DecodeError = e.into();
                    return Some(Err(e.into()));
                }
            }
        }

        return None;
    }
}
impl<I> Id3Decoder<I> {
    pub fn new(paths: I) -> Self {
        Self { paths }
    }
}
