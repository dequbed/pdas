use chrono::prelude::*;

use metaflac;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Audio {
    pub artist: Vec<String>,
    pub title: String,
    pub album: Option<String>,
    pub genre: Option<String>,
    pub lyrics: Option<String>,
    pub published: Option<DateTime>
}
