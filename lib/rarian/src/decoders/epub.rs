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
