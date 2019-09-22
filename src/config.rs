use serde::{Serialize, Deserialize};
use std::path::Path;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
struct Config<'a> {
    path: &'a Path,

    remotes: HashMap<&'a str, Remote<'a>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Remote<'a> {
    url: &'a str,
}

fn read() {
}
