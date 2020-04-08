use clap::ArgMatches;
use slog::Logger;

use rarian::db::dbm::{self, DBManager};
use rarian::db::Database;
use rarian::query::Querier;
use rarian::Transaction;
use rarian::query::parse;

use crate::Settings;

pub async fn query(log: &Logger, s: Settings, m: &ArgMatches<'_>) {
    let target = m.value_of("target").expect("No value for `TARGET` set!");
    let query = m.value_of("query").expect("No value for `QUERY` set!");

    let mut dbmb = DBManager::builder();
    dbmb.set_flags(dbm::EnvironmentFlags::READ_ONLY);
    dbmb.set_max_dbs(126);
    dbmb.set_map_size(10485760);
    let dbm = DBManager::from_builder(&s.databasepath, dbmb).unwrap();

    let txn = dbm.read().unwrap();
    info!(log, "Opening database {}", target);
    let db = match Database::open(&txn, target) {
        Ok(db) => db,
        Err(e) => {
            crit!(log, "Can't open database {}: {:?}", target, e);
            return;
        }
    };

    let mut qr = Querier::new(&txn, &db);
    match parse(query) {
        Ok(q) => {
            match qr.run(q) {
                Ok(matches) => {
                    println!("{:?}", matches);
                },
                Err(e) => {
                    crit!(log, "Failed to run query: {:?}", e);
                }
            }
        },
        Err(e) => {
            crit!(log, "Can't parse query: {:?}", e)
        }
    }

    Transaction::commit(txn).unwrap();
}
