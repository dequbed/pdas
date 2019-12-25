use clap::{App, ArgMatches};

use crate::Librarian;

use rarian::db::{
    EntryDB,
    TitleDB,
};
use rarian::index::Indexer;
use rarian::db::entry::UUID;

pub fn clap() -> App<'static, 'static> {
    clap_app!( @subcommand db =>
        (@setting SubcommandRequiredElseHelp)
        (about: "DB management subsystem")
        (@subcommand read =>
            (about: "read from the specified db")
            (@arg key: * "key to extract"))
        (@subcommand dump =>
            (about: "dump contents of the db"))
        (@subcommand index =>
            (about: "dump the index of the db"))
        (@subcommand search =>
            (about: "search for a word")
            (@arg term: * "search term"))
        (@subcommand create =>
            (about: "create all db files"))
    )
}

pub fn run(lib: Librarian, matches: &ArgMatches) {
    match matches.subcommand() {
        ("create", Some(args)) => {
            //let e = lib.dbm.create_named("entry").unwrap();
            //let e = EntryDB::new(e);
            //let t = lib.dbm.create_named("title").unwrap();
            //let t = TitleDB::new(t);
            //let i = Indexer::new(e, t);

            //let rtx = lib.dbm.read().unwrap();

            //let u = UUID::generate();
            //let entry = EntryT::new();
            //i.index(&mut rtx, u, entry);
        }
        ("archive", Some(args)) => {
            
        }
        _ => {}
    }
}
