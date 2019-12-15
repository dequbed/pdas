use git2::{Repository, Config};
use std::process::Command;
use std::env;
use std::process::exit;
use std::path::Path;
use std::fs;

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

