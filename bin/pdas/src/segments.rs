use std::io::{self, BufRead};

use futures::prelude::*;

/// A stream of segments in a byte stream.
///
/// This stream is created by the `segments` function on types that implement [`BufRead`].
///
/// This type is an generalization of [`std::io::Lines`] over any kind of separator.
///
/// [`BufRead`]: https://doc.rust-lang.org/std/io/trait.BufRead.html
/// [`std::io::Lines`]: https://doc.rust-lang.org/std/io/struct.Lines.html
#[derive(Debug)]
pub struct Segments<R> {
    reader: R,
    sep: u8,
}

impl<R: BufRead> Iterator for Segments<R> {
    type Item = io::Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = Vec::new();
        match self.reader.read_until(self.sep, &mut buf) {
            Ok(0) => None,
            Ok(_n) => {
                if buf.ends_with(&[self.sep]) {
                    buf.pop();
                }
                // FIXME: Not everything is UTF-8, you know?
                let buf = unsafe { String::from_utf8_unchecked(buf) };
                Some(Ok(buf))
            }
            Err(e) => Some(Err(e))
        }
    }
}

pub fn segments<R: BufRead>(reader: R, sep: u8) -> Segments<R> {
    Segments { reader, sep }
}
