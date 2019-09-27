use clap::App;

pub const SUBCOMMAND: &'static str = "setup";

pub fn clap() -> App<'static, 'static> {
    clap_app!( @subcommand setup =>
        (about: "Initialize the pdas repository")
    )
}

