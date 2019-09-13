use clap::{App, ArgMatches};

use directories::ProjectDirs;

use lmdb::Environment;
use lmdb::EnvironmentFlags;
use lmdb::WriteFlags;
use lmdb::Transaction;

pub fn clap() -> App<'static, 'static> {
    clap_app!( @subcommand db =>
        (@setting SubcommandRequiredElseHelp)
        (about: "CouchDB subsystem")
        (@subcommand read =>
            (about: "read from the specified db")
        )
    )
}

pub fn run(matches: &ArgMatches) {
    if let Some(proj_dir) = ProjectDirs::from("org", "Paranoidlabs", "Librarian") {
        let db_path = proj_dir.data_dir();
        std::fs::create_dir(db_path).unwrap();

        let env = Environment::new()
            .set_flags(EnvironmentFlags::MAP_ASYNC | EnvironmentFlags::WRITE_MAP)
            .open(db_path);

        match env {
            Err(e) => {
                error!("Failed to open database environment: {}", e);
            }
            Ok(e) => {
                let db = e.open_db(None).expect("We just opened that...");

                let mut t = e.begin_rw_txn().unwrap();
                t.put(db, &[1], &[1], WriteFlags::empty());

                let v = t.get(db, &[1]);
                println!("{:?}", v);
            }
        }
    }
}
