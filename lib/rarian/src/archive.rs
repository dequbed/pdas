use std::path::{Path, PathBuf};
use std::thread;

use futures::stream::{self, StreamExt};
use futures::future::join;
use futures::executor;
use git_annex;

use crate::Result;

pub fn archive(path: String) -> Result<String> {
    let pathv = vec![path];
    let (f, s) = git_annex::add::add(stream::iter(pathv));

    thread::spawn(move || {
        executor::block_on(f);
    });

    let vf = s.unwrap().collect();
    let mut v: Vec<std::result::Result<(String, String), String>> = executor::block_on(vf);

    let (_f, k) = v.pop().unwrap().unwrap();

    return Ok(k);
}
