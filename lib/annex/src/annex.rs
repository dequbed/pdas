/// The `Annex` object contains all necessary context for future `git-annex` calls
///
/// Multiple structs may be constructed when multiple repositories are being used.
pub struct Annex;

impl Annex {
    pub fn new() -> Self {
        Annex
    }
}
