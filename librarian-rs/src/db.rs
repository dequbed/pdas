use clap::{App, ArgMatches};

use sofa::Client;

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
    let c = Client::new("http://127.0.0.1:5984".to_string()).unwrap();
    println!("{:?}", matches);

}
