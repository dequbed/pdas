use std::path::PathBuf;

use std::collections::HashMap;

use crate::error::Error;
use crate::decoders::Decoder;
use crate::Librarian;
use crate::git;
use rust_stemmers::{Algorithm, Stemmer};

use crate::database::{Key, Metadatabase, RwTransaction, Transaction, Stringindexdb, SHA256E, Occurance};
use crate::storage::MetadataOwned;

pub fn decode<I: Iterator<Item=String>>(lib: Librarian, iter: I) {
    let pb: Vec<std::path::PathBuf> = iter.map(PathBuf::from).collect();

    info!("Decoding files");
    let meta = Decoder::decode(&pb);
    info!("Annexing files");
    let keys = git::annex_add(&pb).unwrap();
    info!("Annexed files");

    let mut combined: Vec<(Key, MetadataOwned)> = Vec::new();
    let mut keymap = HashMap::<&str, Key>::new();

    if !meta.is_empty() {

        let metaf = pb
            .iter()
            .map(|b| b.file_name().and_then(|os| os.to_str()))
            .zip(meta);
        info!("Storing Metadata");

        for (key, file) in keys.iter() {
            keymap.insert(&file, *key);
        }
        for i in metaf {
            match i {
                (Some(p), Ok(f)) => {
                    if let Some(k) = keymap.get(&p) {
                        combined.push((*k, f));
                        keymap.remove(&p);
                    } else {
                        warn!("file {} does not appear to have been annexed or was annexed twice", p);
                    }
                }
                (_, Err(e)) => {
                    error!("Failure to decode: {:?}", e);
                }
                _ => error!("Failure to decode path"),
            }
        }
    }

    if !keymap.is_empty() {
        warn!("Some files have been indexed but no metadata got extracted.");
        if log_enabled!(log::Level::Info) {
            info!("List of files:");
            for k in keymap.keys() {
                info!("    {}", k);
            }
        }
    }

    let db = Metadatabase::new(lib.dbm.create_named("main").unwrap());
    let dbi = Stringindexdb::open(&lib.dbm, "title").unwrap();

    let en_stem = Stemmer::create(Algorithm::English);

    for (k,v) in combined.into_iter() {
        let r = lib.dbm.read().unwrap();
        let mut w = lib.dbm.write().unwrap();
        index(dbi, &r, &mut w, &en_stem, k, &v);
        store(db, &mut w, &k, v);
        w.commit().unwrap();
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
