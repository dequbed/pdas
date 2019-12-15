use rarian;
use rarian::db::EntryDB;
use rarian::db::TitleDB;
use rarian::index::Indexer;
use rarian::DBManager;

use tempfile::{tempdir, TempDir};

fn create_manager() -> (DBManager, TempDir) {
    let dir = tempdir().unwrap();

    let mut dbmb = DBManager::builder();
    dbmb.set_flags(rarian::EnvironmentFlags::MAP_ASYNC | rarian::EnvironmentFlags::WRITE_MAP);
    dbmb.set_map_size(10485760);
    dbmb.set_max_dbs(4);

    (rarian::DBManager::from_builder(dir.path(), dbmb).unwrap(), dir)
}

fn done(dir: TempDir) {
    dir.close().unwrap();
}

#[test]
fn store_ret_test() {
    let (dbm,dbdir) = create_manager();
    let edb = dbm.create_named("entries").unwrap();
    let entries = EntryDB::new(edb);

    done(dbdir);
}
