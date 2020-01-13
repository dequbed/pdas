use std::path::Path;
use std::collections::HashMap;

use rarian::RMDSE;
use rarian::db::entry::{
    UUID,
    EntryT,
    EntryOwn,
    FileT,
};

fn main() {
    let dbdir = Path::new("/tmp/asdf");
    let dbdir2 = Path::new("/tmp/bsdf");

    println!("{:?}", &dbdir);

    let rmdse = RMDSE::open(&dbdir).unwrap();

    rmdse.import(dbdir).unwrap();
    rmdse.export(dbdir2).unwrap();
}
