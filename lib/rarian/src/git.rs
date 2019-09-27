use git2::{Repository, Config};
use std::io;
use std::process::Command;
use std::io::BufReader;
use std::io::BufRead;
use std::io::Write;
use json;
use std::env;
use std::process::Stdio;
use std::thread;
use std::process::exit;
use std::path::{PathBuf, Path};
use std::fs;

use crate::error::Result;
use crate::database::Key;

pub fn annex_add(list: &[PathBuf]) -> Result<Vec<(Key, String)>> {
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
            let k = Key::try_parse(&ks).unwrap();
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

pub fn import_needed<I: Iterator<Item=PathBuf>>(dir: &Path, paths: I) -> Result<Vec<(Key, PathBuf)>> {
    env::set_current_dir(dir)?;

    let cpaths = paths
        .map(|p| p.canonicalize())
        .filter_map(|r| r.ok());

    let mut child = Command::new("git-annex")
        .args(&["import", "--skip-duplicates", "--json", "--json-error-messages"])
        .args(cpaths)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .spawn().unwrap();

    let stdout = child.stdout.take()
        .ok_or(io::Error::new(io::ErrorKind::UnexpectedEof, "child stdout was closed"))?;

    let mut out = Vec::new();

    for line in BufReader::new(stdout).lines() {
        let line = line?;
        if !line.is_empty() {
            let j = json::parse(&line)?;
            out.push(j);
        }
    }

    info!("Read all from git-annex");

    let r = out.into_iter().filter_map(|mut j: json::JsonValue| {
        info!("JSON: {:?}", j);
        match j.remove("note") {
            json::JsonValue::String(note) => { 
                if note.as_str() == "duplicate; skipping" {
                    return None;
                }
            }
            _ => {}
        }

        let ks: String = j.remove("key").take_string().unwrap();
        let k: Key = Key::try_parse(&ks).unwrap();
        let p: String = j.remove("file").take_string().unwrap();
        let f = std::path::Path::new(&p).to_path_buf();
        return Some((k,f));
    }).collect();

    Ok(r)
}

pub fn init(dir: &Path, remotes: &[(String, String)]) {
    if !dir.exists() {
        info!("Creating git directory {}", dir.display());
        fs::create_dir_all(dir).unwrap();
    }

    if let Err(e) = env::set_current_dir(dir) {
        error!("failed to change directory: {:?}", e);
        exit(-1);
    }

    let repo = if !Path::new(".git").exists() {
        Repository::init(dir)
    } else {
        Repository::open(dir)
    };

    let repo = match repo {
        Ok(r) => r,
        Err(e) => {
            error!("failed to initialize git repository: {}", e);
            exit(-1);
        }
    };

    // TODO give repos a description
    cmdrun(Command::new("git-annex")
            .arg("init")
            // Version 7 is default by now but still
            .arg("--version=7"),
        "git-annex init");

    match Config::open(Path::new(".git/config")) {
        Ok(mut config) => {
            config.set_bool("annex.thin", true).expect("Failed to set annex.thin in git config");
        }
        Err(e) => {
            error!("Failed to open git config: {}", e);
            exit(-1);
        }
    }

    cmdrun(Command::new("git-annex")
            .arg("wanted")
            .arg(".")
            .arg("present"),
        "configuring preferred content");
    cmdrun(Command::new("git-annex")
            .arg("untrust")
            .arg("."),
        "untrusting local repository");


    // FIXME: Don't add remotes we already have
    for (name, remote) in remotes.into_iter() {
        if repo.find_remote(&name).is_err() {
            if let Err(e) = repo.remote(&name, &remote) {
                error!("Failed to add remote {}: {}", &name, e)
            }
        }
    }

    cmdrun(Command::new("git-annex").arg("sync"), "git-annex sync");
}

fn cmdrun(command: &mut Command, name: &str) {
    match command.status() {
        Err(e) => {
            error!("Failed to {}: {}", name, e);
            return;
        },
        Ok(exit) => {
            if !exit.success() {
                if let Some(c) = exit.code() {
                    error!("{} returned with error code: {}", name, c);
                } else {
                    error!("{} was killed by a signal", name);
                }
                return;
            }
        }
    }
}
