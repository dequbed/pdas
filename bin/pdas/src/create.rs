use std::io::Read;
use std::fs::File;

use clap;
use slog::Logger;

use rarian::schema::Schema;
use rarian::db::Database;
use rarian::db::dbm::DBManager;
use rarian::Transaction;

use crate::Settings;

pub async fn create(log: &Logger, s: Settings, m: &clap::ArgMatches<'_>) {
    let target = m.value_of("target").expect("No value for `target` set!");
    let schemapath = m.value_of("schema").expect("No value for `schema` set!");

    let mut schemaf = match File::open(schemapath) {
        Ok(f) => f,
        Err(e) => {
            error!(log, "Can't open schema file: {}", e);
            return;
        }
    };
    let mut buf = Vec::new();
    if let Err(e) = schemaf.read_to_end(&mut buf) {
        error!(log, "Failed to read schema file: {}", e);
    }
    let schema = match Schema::from_yaml(&buf[..]) {
        Ok(s) => s,
        Err(e) => {
            error!(log, "Couldn't decode schema file: {:?}", e);
            return;
        }
    };

    let mut dbmb = DBManager::builder();
    dbmb.set_max_dbs(126);
    dbmb.set_map_size(10485760);
    let dbm = DBManager::from_builder(&s.databasepath, dbmb).unwrap();

    let mut txn = dbm.write().unwrap();

    info!(log, "Creating database {}", target);
    match Database::create(&mut txn, target, schema) {
        Ok(db) => db,
        Err(e) => {
            crit!(log, "Can't create database {}: {:?}", target, e);
            return;
        }
    }

    if let Err(e) = Transaction::commit(txn) {
        crit!(log, "Failed to commit transaction: {}", e);
    }
}
