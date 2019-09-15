use crate::error::Error;

use json;

use std::path::PathBuf;

use clap::{App, ArgMatches};

use std::process::{Command, Stdio};
use std::io::{self, BufReader, BufRead, Write};

use std::thread;

use crate::Librarian;
use crate::db::{Key, Backend, SHA256E};

pub const SUBCOMMAND: &'static str = "git";

pub fn clap() -> App<'static, 'static> {
    clap_app!( @subcommand git =>
        (@setting TrailingVarArg)
        (about: "direct git subsystem")
        (@arg cmd: ... "command to forward to git")
    )
}

pub fn run(lib: Librarian, args: &ArgMatches) {
    let mut git = Command::new("git");

    if let Some(vargs) = args.values_of_os("cmd") {
        git.args(vargs);
    }

    match git.spawn() {
        Ok(mut child) => { child.wait().expect("git process failed to start"); },
        Err(e) => { error!("Failed to start `git`: {}", e) },
    }
}

pub fn annex_add(list: &[PathBuf]) -> Result<Vec<(SHA256E, String)>, Error> {
    let mut child = Command::new("git")
        .args(&["annex", "add", "--json", "--json-error-messages", "--batch"])
        .args(&["+RTS", "-N2"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn().unwrap();

    let mut stdin = child.stdin.take()
        .ok_or(io::Error::new(io::ErrorKind::UnexpectedEof, "child stdin was closed"))?;
    let stdout = child.stdout.take()
        .ok_or(io::Error::new(io::ErrorKind::UnexpectedEof, "child stdout was closed"))?;

    let len = list.len();
    let tp = thread::spawn(move || {
        let mut out = Vec::with_capacity(len);

        for line in BufReader::new(stdout).lines() {
            let line = line?;
            if !line.is_empty() {
                let j = json::parse(&line)?;
                out.push(j);
            }
        }

        info!("Read all from annex");

        Ok(out.iter_mut().map(|j: &mut json::JsonValue| {
            let ks: String = j.remove("key").take_string().unwrap();
            let k = SHA256E::try_parse(&ks).unwrap();
            let p: String = j.remove("file").take_string().unwrap();
            let f: String = std::path::Path::new(&p).file_name().and_then(|p: &std::ffi::OsStr| p.to_str().map(str::to_string)).unwrap();
            (k,f)
        }).collect())
    });

    for f in list.iter() {
        stdin.write_all(f.to_str().unwrap().as_bytes())?;
        stdin.write("\n".as_bytes())?;
        stdin.flush()?;
    }

    stdin.flush()?;
    std::mem::drop(stdin);

    child.wait()?;

    tp.join().unwrap()
}
