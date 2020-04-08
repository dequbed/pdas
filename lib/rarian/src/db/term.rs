use libc::size_t;
use lmdb::{
    Database,
    Transaction,
    RwTransaction,
    RoTransaction,
    WriteFlags,
    Iter,
    Cursor,
};
use serde::{
    Deserialize,
    Serialize,
};

use std::collections::{HashSet, HashMap};
use std::borrow::Cow;

use rust_stemmers::{Algorithm, Stemmer};

use crate::error::{Result, Error};

use crate::db::meta::{
    Metakey,
    Metavalue,
};
use crate::db::entry::EntryT;
use crate::uuid::UUID;
use crate::db::{
    EntryDB,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Matches(HashSet<UUID>);

impl Matches {
    pub fn new(set: HashSet<UUID>) -> Self {
        Self ( set )
    }

    pub fn empty() -> Self {
        Self ( HashSet::with_capacity(0) )
    }

    pub fn into_set(self) -> HashSet<UUID> {
        self.0
    }

    pub fn encoded_size(&self) -> Result<u64> {
        bincode::serialized_size(&self.0).map_err(Error::Bincode)
    }

    pub fn encode_into(&self, bytes: &mut [u8]) -> Result<()> {
        bincode::serialize_into(bytes, &self).map_err(Error::Bincode)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes).map_err(Error::Bincode)
    }

    pub fn combine(&mut self, other: &Matches) {
        let union = self.0.union(&other.0);
        self.0 = union.map(|x| *x).collect();
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TermDB {
    db: Database,
}

impl TermDB {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    fn get_bytes<'txn, T: Transaction, K: AsRef<[u8]>>(self, txn: &'txn T, key: &K) -> Result<&'txn [u8]> {
        txn.get(self.db, key).map_err(Error::LMDB)
    }

    fn reserve_bytes<'txn, K: AsRef<[u8]>>(self, txn: &'txn mut RwTransaction, key: &K, len: usize, flags: WriteFlags) -> Result<&'txn mut [u8]> {
        txn.reserve(self.db, key, len as size_t, flags).map_err(Error::LMDB)
    }

    pub fn get<'txn, T: Transaction>(self, txn: &'txn T, key: &str) -> Result<Matches> {
        self.get_bytes(txn, &key)
            .and_then(Matches::decode)
            .or_else(|e| match e {
                Error::LMDB(lmdb::Error::NotFound) => Ok(Matches::empty()),
                e => Err(e),
            })
    }

    pub fn put<'txn>(self, txn: &'txn mut RwTransaction, key: &str, m: Matches) -> Result<()>
    {
        let len = m.encoded_size()? as usize;
        let buf = self.reserve_bytes(txn, &key, len, WriteFlags::empty())?;
        m.encode_into(buf)
    }

    pub fn iter_start<'txn, T: Transaction>(self, txn: &'txn T) -> Result<Iter<'txn>> {
        let mut cursor = txn.open_ro_cursor(self.db)?;
        Ok(cursor.iter_start())
    }

    pub fn insert_match<'txn>(&mut self, txn: &'txn mut RwTransaction, key: &str, uuid: UUID) -> Result<bool> {
        match self.get(txn, key) {
            Ok(matches) => {
                let mut matches = matches.into_set();
                let r = matches.insert(uuid);
                self.put(txn, key, Matches::new(matches))?;

                Ok(r)
            }
            Err(Error::LMDB(lmdb::Error::NotFound)) => {
                let mut matches = HashSet::new();
                let r = matches.insert(uuid);
                self.put(txn, key, Matches::new(matches))?;

                Ok(r)
            }
            Err(e) => return Err(e),
        }
    }

    pub fn insert_matches<'txn>(&mut self, txn: &'txn mut RwTransaction, key: &str, other: &Matches) -> Result<()> {
        match self.get(txn, key) {
            Ok(mut m) => {
                m.combine(other);
                self.put(txn, key, m)?;

                Ok(())
            }
            Err(Error::LMDB(lmdb::Error::NotFound)) => {
                self.put(txn, key, other.clone())?;

                Ok(())
            }
            Err(e) => return Err(e),
        }
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
            self.insert_match(txn, &stem, uuid)?;
        }

        Ok(())
    }

    pub fn list<'txn, T: Transaction>(&self, txn: &'txn T) -> Result<()> {
        let i = self.iter_start(txn)?;

        for r in i {
            if let Ok((k,v)) = r {
                let m = Matches::decode(v)?;
                let k = std::str::from_utf8(k)?;
                println!("{}:\t{:?}", k, m);
            }
        }

        Ok(())
    }

    pub fn lookup<'txn, T: Transaction>(&self, txn: &'txn T, term: &str) -> Result<Matches> {
        let s = Stemmer::create(Algorithm::English);
        let stem = s.stem(term);

        self.get(txn, &stem)
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
