use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::ops::Bound;

use rarian::db::dbm::{self, DBManager};
use rarian::db::Database;
use rarian::db::meta::Metakey;
use rarian::query::{Querier, Query, Filter};
use rarian::schema;

fn main() {
    let dbdir = Path::new("/tmp/asdf");
    let dbdir2 = Path::new("/tmp/bsdf");

    let mut fp = File::open("/tmp/asdf/schema.yml").unwrap();
    let mut buf = Vec::new();
    fp.read_to_end(&mut buf).unwrap();
    let schema = schema::from_yaml(&buf).unwrap();
    println!("{:?}", schema);

    let mut dbmb = DBManager::builder();
    dbmb.set_flags(dbm::EnvironmentFlags::MAP_ASYNC | dbm::EnvironmentFlags::WRITE_MAP);
    dbmb.set_max_dbs(126);
    dbmb.set_map_size(10485760);
    let dbm = DBManager::from_builder(dbdir.as_ref(), dbmb).unwrap();

    Database::create(&dbm, "img", schema).unwrap();

    let mut db = Database::open(&dbm, "img").unwrap();
    db.import(dbdir).unwrap();
    db.dump().unwrap();

    let rtxn = dbm.read().unwrap();
    let mut qr = Querier::new(rtxn, &db);
    let q1 = Query::F(Filter::TermExists("asdf"), Metakey::Description);
    let query = Query::AND(
        &q1,
        &Query::F(Filter::IntInRange(Bound::Unbounded, Bound::Included(1555538400)), Metakey::Date)
    );

    let s = qr.run(q1).unwrap();
    println!("Query {:?} results: {:?}", q1, s);
    let s = qr.run(query).unwrap();
    println!("Query {:?} results: {:?}", query, s);

    db.close().unwrap();
}
