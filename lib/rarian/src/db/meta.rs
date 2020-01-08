use std::collections::{HashMap, HashSet};

use serde::{Serialize, Deserialize};

pub type Metakey = u32;

pub trait Metavalue<'de> {
    fn decode(bytes: &'de [u8]) -> Self;
}

impl<'de> Metavalue<'de> for &'de str {
    fn decode(bytes: &'de [u8]) -> Self {
        unsafe { std::str::from_utf8_unchecked(bytes) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_metadata_test() {
        let title = "testtitle".to_string();
        let author = "testauthor".to_string();
        let filename = "testfilename".to_string();
        let subject = "testsubject".to_string().into_boxed_str().into_boxed_bytes();
        let filesize = 361567;
        let mut metamap = HashMap::new();
        metamap.insert(Metakey::Subject, subject);
        let m = MetadataOwned::new(title, Some(author), filename, Some(filesize), metamap);
        println!("{:?}", m);

        let l = m.encoded_size().unwrap() as usize;
        let mut vec: Vec<u8> = Vec::with_capacity(l);
        unsafe { vec.set_len(l) };
        let mut vec = vec.into_boxed_slice();
        m.encode_into(&mut vec[..l]).unwrap();

        let n = Metadata::decode(&vec[..l]).unwrap();

        assert_eq!(n.title, m.title);
        assert_eq!(n.author.map(|s| s.to_string()), m.author);
        assert_eq!(n.filename, m.filename);
        assert_eq!(n.filesize, m.filesize);
    }
}
