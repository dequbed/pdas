use std::collections::{HashSet, HashMap};
use std::borrow::Cow;

use serde::{Serialize, Deserialize};
use rust_stemmers::{Algorithm, Stemmer};
use lmdb::{
    RwTransaction,
};

use crate::db::meta::{
    Metakey,
    Metavalue,
};
use crate::error::{Error, Result};
use crate::db::entry::{UUID, EntryT};
use crate::db::{
    EntryDB,
    TermDB,
};

pub enum Index {
    Term(TermIndex)
}
impl Index {
    pub fn index<'txn>(&mut self, txn: &'txn mut RwTransaction, v: &[u8], uuid: UUID) -> Result<()> {
        match self {
            Index::Term(ti) => {
                let t = TermIndex::decode(v);
                ti.index(txn, t, uuid)
            }
        }
    }
}

pub struct TermIndex {
    termdb: TermDB,
}

impl TermIndex {
    pub fn new(termdb: TermDB) -> Self {
        Self { termdb }
    }

    pub fn index<'txn>(&mut self, txn: &'txn mut RwTransaction, term: String, uuid: UUID) -> Result<()> {
        let s = Stemmer::create(Algorithm::English);

        let title = term.to_lowercase();
        let words = title.split_whitespace();
        let wordsc = words.map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()));
        let wordstems = wordsc.map(|w| s.stem(w));

        let fillwords = wordstems.filter(|s| !is_stopword(s));
        let filtered = fillwords.filter(|s| !s.is_empty());

        for stem in filtered {
            self.termdb.insert_match(txn, &stem, uuid)?;
        }

        Ok(())
    }

    pub fn decode(buf: &[u8]) -> String {
        unsafe { std::str::from_utf8_unchecked(buf).to_string() }
    }
}

pub struct Indexer {
    entrydb: EntryDB,
    indexer: HashMap<Metakey, Index>,
}

impl Indexer {
    pub fn new(entrydb: EntryDB, indexer: HashMap<Metakey, Index>) -> Self {
        Indexer { 
            entrydb,
            indexer,
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
        for (k,v) in entry.metadata().iter() {
            if let Some(i) = self.indexer.get_mut(k) {
                i.index(txn, v.as_ref(), uuid)?;
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
