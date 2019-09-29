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

pub fn import<'a, I: Iterator<Item=&'a Path>>(dir: &Path, paths: I) -> Result<Vec<(Key, PathBuf)>> {
    env::set_current_dir(dir)?;

    // TODO chunk here because a command line can only be so long.
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


// Import process
// 1. Check if the file is already annexed, if so => SKIP
// 1.1 git-annex calckey
// 2. Else read metadata, annex the file into the object store
// 3. Calculate the path, move the link
// 4. Do 1-3 for all files, then commit & sync
//
// Output of git-annex import:
// When skipping because a file was already annexed:
// {"command":"import","note":"duplicate; skipping","success":true,"file":"USB Type-C Specification Release 1.1.pdf"}
//
// When importing a file:
// {"command":"import","success":true,"key":"SHA256E-s562633--bf8560a3fe8ff8e87dffa1b0d1caf868fb64862018d2377998ad087dd1631a13.pdf","file":"USB-C_Source_Power_Test_Specification_2018_06_01.pdf"}
//
// When failing because called with --duplicate but file was already annexed:
// {"command":"import","success":false,"file":"USB-C_Source_Power_Test_Specification_2018_06_01.pdf"}
//
// When importing with --skip-duplicates defined:
// {"command":"import","success":true,"key":"SHA256E-s8692742--70e9ef5ae0e8d53933740f5a67326db0178e10f196b324e5ba1cf49956ebb5eb.1.pdf","file":"USB Type-C Specification Release 1.1.pdf"}
//
// Importing three files, one duplicate with `git-annex import --skip-duplicates --json --json-error-messages`
// {"command":"import","note":"duplicate; skipping","success":true,"error-messages":[],"file":"Delay.flac"}
// {"command":"import","success":true,"key":"SHA256E-s32643489--08e9e295f3d094a3ba3fc32a509a89a1074cfa1e6ff7fbe83aace3d87e96f36c.flac","error-messages":[],"file":"Dust.flac"}
// {"command":"import","success":true,"key":"SHA256E-s43993918--775c60564d2937794f508674a0670196aecbcc797f7bd38ff7889c1be9bfc64d.flac","error-messages":[],"file":"Plus Four.flac"}
