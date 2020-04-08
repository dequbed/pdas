use std::collections::HashMap;

use std::path::PathBuf;

use clap;
use slog::Logger;

use serde::Deserialize;

use rarian::db::Database;
use rarian::db::entry::{EntryT, FileT};
use rarian::db::meta::{Metakey, Metavalue};
use rarian::db::dbm::{self, DBManager};
use rarian::Transaction;

use crate::Settings;

use futures::prelude::*;

pub async fn export(log: &Logger, s: Settings, m: &clap::ArgMatches<'_>) {
    let target = m.value_of("target").expect("No value for `TARGET` set!");
    let entries = m.value_of("entries").expect("No entries folder provided");

    let mut dbmb = DBManager::builder();
    dbmb.set_flags(dbm::EnvironmentFlags::empty());
    dbmb.set_max_dbs(126);
    dbmb.set_map_size(10485760);
    let dbm = DBManager::from_builder(&s.databasepath, dbmb).unwrap();

    let mut txn = dbm.read().unwrap();
    info!(log, "Opening database {}", target);
    let mut db = match Database::open(&txn, target) {
        Ok(db) => db,
        Err(e) => {
            crit!(log, "Can't open database {}: {:?}", target, e);
            return;
        }
    };

    let entries = PathBuf::from(entries.to_string());

    if let Err(e) = db.export_with(&entries, &txn) {
        error!(log, "Failed to export entries: {:?}", e);
    }

    Transaction::commit(txn);
}
