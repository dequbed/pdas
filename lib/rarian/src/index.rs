use std::collections::HashSet;
use std::borrow::Cow;

use serde::{Serialize, Deserialize};
use rust_stemmers::{Algorithm, Stemmer};
use lmdb::{
    RwTransaction,
};

use crate::storage::{
    Metakey,
    Meta,
    Title,
};
use crate::error::{Error, Result};
use crate::db::entry::{UUID, EntryT};
use crate::db::{
    EntryDB,
    TitleDB,
};

pub struct Indexer {
    entrydb: EntryDB,
    titledb: TitleDB,
}

impl Indexer {
    pub fn new(entrydb: EntryDB, titledb: TitleDB) -> Self {
        Indexer { 
            entrydb,
            titledb,
        }
    }

    pub fn index<'txn, B>
        ( &mut self
        , txn: &'txn mut RwTransaction
        , uuid: UUID
        , entry: EntryT<B>
        )
        -> Result<()>
        where B: Serialize + Deserialize<'txn> + AsRef<[u8]>
    {
        let st = Stemmer::create(Algorithm::English);
        self.index_st(&st, txn, uuid, entry)
    }

    pub fn index_st<'txn, B>
        ( &mut self
        , s: &Stemmer
        , txn: &'txn mut RwTransaction
        , uuid: UUID
        , entry: EntryT<B>
        )
        -> Result<()>
        where B: Serialize + Deserialize<'txn> + AsRef<[u8]>
    {
        for (k,v) in entry.metadata().iter() {
            match k {
                Metakey::Title => {
                    let title = Title::decode(v.as_ref());
                    let title = title.to_lowercase();
                    let words = title.split_whitespace();
                    let wordsc = words.map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()));
                    let wordstems = wordsc.map(|w| s.stem(w));

                    let fillwords = wordstems.filter(|s| !is_stopword(s));
                    let filtered = fillwords.filter(|s| !s.is_empty());

                    for stem in filtered {
                        self.titledb.insert_match(txn, &stem, uuid)?;
                    }
                },
                _ => {}
            }
        }

        self.entrydb.put(txn, &uuid, entry)?;

        Ok(())
    }
}

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
