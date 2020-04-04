use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::ops::Bound;

use rarian::db::dbm::{self, DBManager};
use rarian::db::Database;
use rarian::db::meta::Metakey;
use rarian::query::{Querier, Query, Filter, parse};
use rarian::schema;

fn main() {
    let dbdir = Path::new("/tmp/asdf");
    let dbdir2 = Path::new("/tmp/bsdf");

    let mut fp = File::open("/tmp/asdf/schema.yml").unwrap();
    let mut buf = Vec::new();
    fp.read_to_end(&mut buf).unwrap();

    let mut dbmb = DBManager::builder();
    dbmb.set_flags(dbm::EnvironmentFlags::MAP_ASYNC | dbm::EnvironmentFlags::WRITE_MAP);
    dbmb.set_max_dbs(126);
    dbmb.set_map_size(10485760);
    let dbm = DBManager::from_builder(dbdir.as_ref(), dbmb).unwrap();

    //Database::create(&dbm, "img", schema).unwrap();

    //let mut db = Database::open(&dbm, "img").unwrap();
    //db.import(dbdir).unwrap();
    //db.dump().unwrap();

    //let rtxn = dbm.read().unwrap();
    //let mut qr = Querier::new(rtxn, &db);
    let q1 = parse("description:asdf").unwrap();
    let s = "test description:asdf title:05 date:[1557784800..]";
    let query = parse(&s).unwrap();
    println!("{} equals \"{:?}\"", s, query);

    //let s = qr.run(q1.clone()).unwrap();
    //println!("Query {:?} results: {:?}", q1, s);
    //let s = qr.run(query.clone()).unwrap();
    //println!("Query {:?} results: {:?}", query, s);

    //db.close().unwrap();
}
