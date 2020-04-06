use serde::{Serialize, Deserialize};

/// The `Annex` object contains all necessary context for future `git-annex` calls
///
/// Multiple structs may be constructed when multiple repositories are being used.
pub struct Annex;

impl Annex {
    pub fn new() -> Self {
        Annex
    }
}


#[derive(Copy,Clone,Debug,Serialize,Deserialize)]
pub struct CommandOutput<'s> {
    pub command: &'s str,
    pub note: &'s str,
    pub success: bool,
    pub key: &'s str,
    pub file: &'s str,
}
