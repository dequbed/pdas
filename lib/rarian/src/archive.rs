use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::Error;
use crate::git;
use rust_stemmers::{Algorithm, Stemmer};

use crate::database::{Key, Metadatabase, RwTransaction, Transaction, Stringindexdb, SHA256E, Occurance};
use crate::storage::MetadataOwned;

/// Imports files into the annex and moves them where they belong according to Metadata
pub fn import<'a, I: Iterator<Item=&'a Path>>(repodir: &Path, iter: I) {
    let randomdirname = format!("import-{:x}", rand::random::<u32>());
    let randomdir = Path::new(&randomdirname);
    let importdir = repodir.join(randomdir);

    if let Ok(v) = git::import(&importdir, iter) {
        let (k,p): (Vec<Key>, Vec<PathBuf>) = v.into_iter().unzip();
    }
}

fn index<T: Transaction>(db: Stringindexdb, r: &T, w: &mut RwTransaction, s: &Stemmer, key: SHA256E, val: &MetadataOwned) {
    let title = val.title.to_lowercase();
    let words = title.split_whitespace();
    let wordsc = words.map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()));
    let wordstems = wordsc.map(|w| s.stem(w));

    let fillwords = wordstems.filter(|s| !is_stopword(s));
    let filtered = fillwords.filter(|s| !s.is_empty());

    let mut set = HashMap::<String, Occurance>::new();
    let mut map = HashMap::<String, Vec<u32>>::new();
    


    for (pos, stem) in filtered.enumerate() {
        let t = &stem;

        match db.get(r, t) {
            Err(Error::LMDB(lmdb::Error::NotFound)) => {}
            Ok(mut o) => {
                // If there is an entry for us, remove it, if not, don't care.
                o.0.remove(&key);
                // We definitely need to later on extend this with our entry, to save it.
                set.insert(stem.to_string(), o);
            },
            Err(e) => {
                error!("while reading index: {:?}", e);
                break;
            },
        }

        map.entry(t.to_string())
            .and_modify(|v| v.push(pos as u32))
            .or_insert(vec![pos as u32]);
    }

    println!("{:?}", set);
    println!("{:?}", map);

    for (term, v) in map.into_iter() {
        let o: Occurance;
        match set.remove(&term) {
            Some(mut oc) => {
                // Occurance in the set means there are other objects that are listed in this
                // index, we need to UPDATE and then write
                oc.0.insert(key, v);
                o = oc;
            },
            None => {
                // There is no entry for this term in the DB => create onew
                let mut map = HashMap::new();
                map.insert(key, v);
                o = Occurance(map);
            }
        }
        db.put(w, &term, &o).unwrap();
    }
}

use std::collections::HashSet;

lazy_static! {
    static ref STOPWORDS: HashSet<&'static str> = {
        let words: &[&'static str] = &["a","able","about","across","after","all","almost","also","am","among","an","and","any","are","as","at","be","because","been","but","by","can","cannot","could","dear","did","do","does","either","else","ever","every","for","from","get","got","had","has","have","he","her","hers","him","his","how","however","i","if","in","into","is","it","its","just","least","let","like","likely","may","me","might","most","must","my","neither","no","nor","not","of","off","often","on","only","or","other","our","own","rather","said","say","says","she","should","since","so","some","than","that","the","their","them","then","there","these","they","this","tis","to","too","twas","us","wants","was","we","were","what","when","where","which","while","who","whom","why","will","with","would","yet","you","your"];
        let mut set = HashSet::new();
        for w in words.iter() {
            set.insert(*w);
        }

        set
    };
}

fn is_stopword(word: &str) -> bool {
    STOPWORDS.contains(word)
}

fn store(db: Metadatabase, w: &mut RwTransaction, key: &Key, val: MetadataOwned) {
    db.put(w, key, val).unwrap();
}
