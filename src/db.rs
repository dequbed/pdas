use std::path::Path;
use std::slice;

use clap::{App, ArgMatches};

use lmdb::Cursor;
use lmdb::Environment;
use lmdb::EnvironmentBuilder;
pub use lmdb::EnvironmentFlags;
use lmdb::WriteFlags;
use lmdb::DatabaseFlags;
use lmdb::Transaction;
use lmdb::{RoCursor, RwCursor};
use lmdb::{RoTransaction, RwTransaction};

use crate::Librarian;
use crate::error::Error;

use crate::decoders::Storables;

pub fn clap() -> App<'static, 'static> {
    clap_app!( @subcommand db =>
        (@setting SubcommandRequiredElseHelp)
        (about: "DB management subsystem")
        (@subcommand read =>
            (about: "read from the specified db")
            (@arg key: * "key to extract")
        )
        (@subcommand dump =>
            (about: "dump contents of the db")
        )
    )
}

pub fn run(lib: Librarian, matches: &ArgMatches) {
    let db = lib.dbm.open().unwrap();

    match matches.subcommand() {
        ("read", Some(args)) => {
            let r = lib.dbm.read().unwrap();

            if let Some(k) = args.value_of("key") {
                if let Some(k) = Key::<SHA256E>::try_parse(k) {
                    match db.get(&r, k) {
                        Ok(r) => println!("{:?}", r),
                        Err(e) => error!("Error: {:?}", e),
                    }
                } else {
                    error!("Invalid key");
                }
            } else {
                // Clap errors out before this gets hit due to setting SubcommandRequiredElseHelp
                unreachable!();
            }
        }
        ("dump", _) => {
            let r = lib.dbm.read().unwrap();

            let c = db.iter_start(&r).unwrap();

            for i in c {
                println!("{:?}", i);
            }
        }
        _ => {}
    }
}

pub trait Backend: Sized + Copy {
    type Store: AsRef<[u8]>;
    fn into_inner(self) -> Self::Store;
    fn try_parse(s: &str) -> Option<Self>;
    fn construct(k: &[u8]) -> Option<Self>;
    fn new() -> Self;

    const NAME: &'static str;
}

use serde::{Serialize, Deserialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SHA256E {
    inner: [u8; 32],
}
impl Backend for SHA256E {
    type Store = [u8; 32];

    fn new() -> Self {
        SHA256E { inner: [0; 32] }
    }

    fn into_inner(self) -> Self::Store {
        self.inner
    }

    fn construct(k: &[u8]) -> Option<Self> {
        if k.len() >= 32 {
            let mut inner: [u8; 32] = Default::default();
            inner.copy_from_slice(&k[0..32]);
            Some(Self { inner })
        } else {
            None
        }
    }

    fn try_parse(s: &str) -> Option<Self> {
        let mut si = s.split("--");
        let [a,b]: [&str; 2] = [si.next().unwrap(), si.next().unwrap()];
        let mut info = a.split('-');
        if let Some(m) = info.next() {
            if m == Self::NAME {
                if let Some(k) = b.split('.').next() {
                    let mut inner = [0u8;32];

                    for (idx, pair) in k.as_bytes().chunks(2).enumerate() {
                        inner[idx] = val(pair[0]) << 4 | val(pair[1])
                    }

                    return Some(Self { inner });
                }
            }
        }

        None
    }

    const NAME: &'static str = "SHA256E";
}

fn val(c: u8) -> u8 {
    match c {
        b'A'...b'F' => c - b'A' + 10,
        b'a'...b'f' => c - b'a' + 10,
        b'0'...b'9' => c - b'0',
        _ => 0
    }
}

use std::marker::PhantomData;
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key<T> {
    phantom: PhantomData<T>,
}

impl<T: Backend> Key<T> {
    fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }
    fn try_parse(s: &str) -> Option<T> {
        T::try_parse(s)
    }
    const NAME: &'static str = T::NAME;
}

use crate::config::Mech;

impl<T> Key<T> {
    pub fn lookup(name: Mech) -> Option<Key<impl Backend>> {
        match name {
            Mech::SHA256E => Some(Key::<SHA256E>::new()),
        }
    }
}

pub struct Manager {
    env: Environment
}

impl Manager {
    pub fn builder() -> EnvironmentBuilder {
        Environment::new()
    }

    pub fn from_builder(path: &Path, env: EnvironmentBuilder) -> Result<Self, Error> {
        Ok(Self {
            env: env.open(path).map_err(Error::LMDB)?
        })
    }

    pub fn open(&self) -> Result<ItemDatabase, Error> {
        self.env.open_db(None).map_err(Error::LMDB).map(ItemDatabase::new)
    }

    pub fn create(&self) -> Result<ItemDatabase, Error> {
        self.env.create_db(None, DatabaseFlags::empty()).map_err(Error::LMDB).map(ItemDatabase::new)
    }

    pub fn read(&self) -> Result<Reader, Error> {
        Ok(Reader::new(self.env.begin_ro_txn().map_err(Error::LMDB)?))
    }

    pub fn write(&self) -> Result<Writer, Error> {
        Ok(Writer::new(self.env.begin_rw_txn().map_err(Error::LMDB)?))
    }
}

pub struct Reader<'env>(pub RoTransaction<'env>);
pub struct Writer<'env>(pub RwTransaction<'env>);

pub trait ReadTransaction {
    fn get<K: AsRef<[u8]>, D, T>(&self, db: lmdb::Database, k: &K, op: D) -> Result<T, Error>
        where D: Fn(&[u8]) -> Result<T, Error>;
    fn open_ro_cursor(&self, db: lmdb::Database) -> Result<RoCursor, Error>;
}

impl<'env> Reader<'env> {
    pub fn new(txn: RoTransaction<'env>) -> Self {
        Reader(txn)
    }

    pub fn abort(self) {
        self.0.abort();
    }
}

impl<'env> ReadTransaction for Reader<'env> {
    fn get<K: AsRef<[u8]>, D, T>(&self, db: lmdb::Database, k: &K, op: D) -> Result<T, Error>
        where D: Fn(&[u8]) -> Result<T, Error>
    {
        let bytes = self.0.get(db, &k)?;
        op(bytes)
    }

    fn open_ro_cursor(&self, db: lmdb::Database) -> Result<RoCursor, Error> {
        self.0.open_ro_cursor(db).map_err(Error::LMDB)
    }
}

impl<'env> Writer<'env> {
    pub fn new(txn: RwTransaction<'env>) -> Self {
        Writer(txn)
    }

    pub fn commit(self) -> Result<(), Error> {
        self.0.commit().map_err(Error::LMDB)
    }

    pub fn abort(self) {
        self.0.abort();
    }

    pub fn put<K: AsRef<[u8]>, V: AsRef<[u8]>>(&mut self, db: lmdb::Database, key: K, value: V, flags: WriteFlags) -> Result<(), Error> {
        self.0.put(db, &key, &value, flags)?;
        Ok(())
    }

    pub fn delete<K: AsRef<[u8]>>(&mut self, db: lmdb::Database, key: K, value: Option<&[u8]>) -> Result<(), Error> {
        self.0.del(db, &key, value).map_err(Error::LMDB)
    }

    pub fn clear(&mut self, db: lmdb::Database) -> Result<(), Error> {
        self.0.clear_db(db).map_err(Error::LMDB)
    }
}


#[derive(Copy, Clone)]
pub struct ItemDatabase {
    db: lmdb::Database,
}

impl ItemDatabase {
    pub fn new(db: lmdb::Database) -> Self {
        Self { db }
    }

    pub fn get<R: ReadTransaction, B: Backend>(self, reader: &R, k: B) -> Result<Storables, Error> {
        reader.get(self.db, &k.into_inner(), |b| bincode::deserialize(b).map_err(Error::Bincode))
    }

    pub fn put<B: Backend>(self, writer: &mut Writer, k: &B, v: &Storables) -> Result<(), Error> {
        // TODO: Allocate memory in LMDB, write directly into it.
        let vec = bincode::serialize(v)?;
        writer.put(self.db, k.into_inner(), &vec, WriteFlags::empty())
    }

    pub fn delete<B: Backend>(self, writer: &mut Writer, k: &B, v: Vec<u8>) -> Result<(), Error> {
        writer.delete(self.db, k.into_inner(), Some(&v))
    }

    pub fn iter_start<R: ReadTransaction>(self, reader: &R) -> Result<Iter, Error> {
        let mut cursor = reader.open_ro_cursor(self.db)?;
        let iter = cursor.iter();
        Ok(Iter {
            iter, 
            cursor,
        })
    }

    pub fn iter_from<R: ReadTransaction, B: Backend>(self, reader: &R, k: B) -> Result<Iter, Error> {
        let mut cursor = reader.open_ro_cursor(self.db)?;
        let iter = cursor.iter_from(k.into_inner());
        Ok(Iter {
            iter, 
            cursor,
        })
    }
}

pub struct Iter<'env> {
    iter: lmdb::Iter<'env>,
    cursor: RoCursor<'env>,
}

impl<'env> Iterator for Iter<'env> {
    type Item = Result<(SHA256E, Storables), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            None => None,
            Some(Ok((k,v))) => {
                match bincode::deserialize(v) {
                    Err(e) => Some(Err(e.into())),
                    Ok(v) => Some(Ok((SHA256E::construct(k).unwrap(),v)))
                }
            }
            Some(Err(e)) => Some(Err(e.into()))
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TermOccurance {
    /// A term occurs in this key
    key: SHA256E,
    /// The term occurs at the n-th positions.
    occ: Vec<u32>,
}

#[derive(Clone, Debug)]
pub enum Term {
    String(String)
}
impl AsRef<[u8]> for Term {
    fn as_ref(&self) -> &[u8] {
        match self {
            Term::String(s) => s.as_bytes(),
        }
    }
}

pub struct IterDup<'env> {
    iter: lmdb::IterDup<'env>,
    cursor: RoCursor<'env>,
}

#[derive(Copy, Clone)]
pub struct TermDatabase {
    db: lmdb::Database,
}

impl TermDatabase {
    pub fn new(db: lmdb::Database) -> Self {
        Self { db }
    }

    pub fn get<R: ReadTransaction>(self, reader: &R, k: Term) -> Result<TermOccurance, Error> {
        reader.get(self.db, &k, |b| bincode::deserialize(b).map_err(Error::Bincode))
    }

    pub fn put(self, writer: &mut Writer, k: Term, v: TermOccurance) -> Result<(), Error> {
        let vec = bincode::serialize(&v)?;
        writer.put(self.db, &k, &vec, WriteFlags::empty())
    }

    pub fn delete(self, writer: &mut Writer, k: Term, v: TermOccurance) -> Result<(), Error> {
        let vec = bincode::serialize(&v)?;
        writer.delete(self.db, &k, Some(&vec))
    }

    pub fn iter_dup_from<R: ReadTransaction>(self, reader: &R, k: Term) -> Result<IterDup, Error> {
        let mut cursor = reader.open_ro_cursor(self.db)?;
        let iter = cursor.iter_dup_from(&k);
        Ok(IterDup {
            iter,
            cursor,
        })
    }
}
