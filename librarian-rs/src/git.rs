use crate::error::Error;

use json;

use std::path::PathBuf;

use clap::{App, ArgMatches};

use std::process::{Command, Stdio};
use std::io::{self, BufReader, BufRead, Write};

pub fn clap() -> App<'static, 'static> {
    clap_app!( @subcommand git =>
        (@setting TrailingVarArg)
        (about: "direct git subsystem")
        (@arg cmd: ... "command to forward to git")
    )
}

pub fn run(args: &ArgMatches) {
    let mut git = Command::new("git");

    if let Some(vargs) = args.values_of_os("cmd") {
        git.args(vargs);
    }

    match git.spawn() {
        Ok(mut child) => { child.wait().expect("git process failed to start"); },
        Err(e) => { error!("Failed to start `git`: {}", e) },
    }
}


pub fn annex_add(list: &[PathBuf]) -> Result<Vec<(String, String)>, Error> {
    let mut child = Command::new("git")
        .args(&["annex", "add", "--json", "--json-error-messages", "--batch"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn().unwrap();

    let stdin = child.stdin.as_mut()
        .ok_or(io::Error::new(io::ErrorKind::UnexpectedEof, "child stdin was closed"))?;

    for f in list.iter() {
        info!("Forwarding to annex: {}", f.display());
        stdin.write_all(f.to_str().unwrap().as_bytes())?;
        stdin.write("\n".as_bytes())?;
        stdin.flush()?;
    }

    stdin.flush()?;

    std::mem::drop(stdin);

    info!("Forwarded all to annex. Waiting for it to finish.");

    child.wait()?;

    info!("annex exited sucessfully");

    let mut out = Vec::with_capacity(list.len());

    let stdout = child.stdout.as_mut()
        .ok_or(io::Error::new(io::ErrorKind::UnexpectedEof, "child stdout was closed"))?;

    for line in BufReader::new(stdout).lines() {
        let line = line?;
        let j = json::parse(&line)?;
        out.push(j);
    }

    info!("Read all from annex");

    Ok(out.iter_mut().map(|j: &mut json::JsonValue| {
        let k: String = j.remove("key").take_string().unwrap();
        let p: String = j.remove("file").take_string().unwrap();
        let f: String = std::path::Path::new(&p).file_name().and_then(|p: &std::ffi::OsStr| p.to_str().map(str::to_string)).unwrap();
        (k,f)
    }).collect())
}
