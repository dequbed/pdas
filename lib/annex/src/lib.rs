#[macro_use]
extern crate log;

//mod git;

pub mod annex;
pub use annex::Annex;
pub mod add;
pub mod init;
