use tempfile::{tempdir, TempDir};

use std::collections::HashMap;

use rarian::RMDSE;
use rarian::db::entry::{
    UUID,
    EntryT,
    EntryOwn,
    FileT,
};

fn create_tmpdir() -> TempDir {
    tempdir().unwrap()
}

fn done(dir: TempDir) {
    dir.close().unwrap();
}

#[test]
fn store_ret_test() {
    let dbdir = create_tmpdir();
    let rmdse = RMDSE::open(&dbdir).unwrap();


    let mut indexer = rmdse.indexer().unwrap();
    let uuid = UUID::generate();
    let entry = EntryOwn::new(FileT::new("BLAKE2B512-s95265--4a9f58b219934a6bcaee8f6adc9a31b5e81c7759112ee4d496c6fd809d60f4677348db9347cf56a25b54b6e209a1cdace04c467987fed46126f14193b9d3ae2b".to_string(), HashMap::new()), HashMap::new());

    indexer.index(uuid, &entry).unwrap();
    indexer.commit().unwrap();

    let mut query = rmdse.query().unwrap();
    let e = query.retrieve(uuid).unwrap();

    assert!(EntryT::meta_ref_eq(&entry, &e));

    done(dbdir);
}
