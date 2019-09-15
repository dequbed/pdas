use std::path::Path;

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
                match db.get_store(&r, k) {
                    Ok(r) => println!("V: {:?}", r),
                    Err(e) => println!("Error: {:?}", e),
                }
            } else {
                println!("No key! D:");
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

    pub fn open(&self) -> Result<Database, Error> {
        self.env.open_db(None).map_err(Error::LMDB).map(Database::new)
    }

    pub fn create(&self) -> Result<Database, Error> {
        self.env.create_db(None, DatabaseFlags::empty()).map_err(Error::LMDB).map(Database::new)
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
pub struct Database {
    db: lmdb::Database,
}

impl Database {
    pub fn new(db: lmdb::Database) -> Self {
        Self { db }
    }

    pub fn get<R: ReadTransaction, K: AsRef<[u8]>, D, T>(self, reader: &R, k: K, decoder: D) -> Result<T, Error> 
        where D: Fn(&[u8]) -> Result<T, Error>
    {
        reader.get(self.db, &k, decoder)
    }

    pub fn get_store<R: ReadTransaction, K: AsRef<[u8]>>(self, reader: &R, k: K) -> Result<Storables, Error> {
        self.get(reader, k, |b| bincode::deserialize(b).map_err(Error::Bincode))
    }

    pub fn put<K: AsRef<[u8]>, T: AsRef<[u8]>>(self, writer: &mut Writer, k: K, v: T) -> Result<(), Error> {
        writer.put(self.db, &k, &v, WriteFlags::empty())
    }

    pub fn put_store<K: AsRef<[u8]>>(self, writer: &mut Writer, k: K, v: &Storables) -> Result<(), Error> {
        // TODO: Allocate memory in LMDB, write directly into it.
        let vec = bincode::serialize(v)?;

        self.put(writer, k, &vec)
    }

    pub fn delete<K: AsRef<[u8]>>(self, writer: &mut Writer, k: K, v: Vec<u8>) -> Result<(), Error> {
        writer.delete(self.db, &k, Some(&v))
    }

    pub fn iter_start<R: ReadTransaction>(self, reader: &R) -> Result<Iter, Error> {
        let mut cursor = reader.open_ro_cursor(self.db)?;
        let iter = cursor.iter();
        Ok(Iter {
            iter, 
            cursor,
        })
    }

    pub fn iter_from<R: ReadTransaction, K: AsRef<[u8]>>(self, reader: &R, k: K) -> Result<Iter, Error> {
        let mut cursor = reader.open_ro_cursor(self.db)?;
        let iter = cursor.iter_from(k);
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
    type Item = Result<(&'env str, Storables), Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            None => None,
            Some(Ok((k,v))) => {
                match bincode::deserialize(v) {
                    Err(e) => Some(Err(e.into())),
                    Ok(v) => Some(Ok((std::str::from_utf8(k).unwrap(),v)))
                }
            }
            Some(Err(e)) => Some(Err(e.into()))
        }
    }
}
