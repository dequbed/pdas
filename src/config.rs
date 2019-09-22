use serde::{Serialize, Deserialize};
use std::io::{Read, Write};
use std::path::{PathBuf, Path};
use std::collections::HashMap;
use std::fs::{self, File};
use std::default::Default;
use directories::{ProjectDirs, UserDirs};

use crate::error::{Result, Error};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Config {
    pub path: PathBuf,

    pub remotes: HashMap<String, Remote>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            path: default_path(),
            remotes: HashMap::new(),
        }
    }
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Remote {
    pub url: String,
}

fn default_path() -> PathBuf {
    if let Some(d) = ProjectDirs::from("org", "Paranoidlabs", "pdas") {
        d.data_dir().into()
    } else if let Some(d) = UserDirs::new() {
        d.home_dir().join(".pdas")
    } else {
        error!("could not find a valid config or home directory, aborting.");
        std::process::exit(-1);
    }
}

pub fn read_or_create(path: Option<&str>) -> Result<Config> {
    let mut cf: File;
    if let Some(o) = path {
        cf = File::open(o)?;
    } else {
        if let Some(d) = ProjectDirs::from("org", "Paranoidlabs", "pdas") {
            let dir = d.config_dir();
            if !Path::exists(dir) {
                fs::create_dir_all(dir)?;
            }

            let path = dir.join("config.toml");

            println!("dir: {}, path: {}", dir.display(), path.display());

            if !Path::exists(&path) {
                let mut f = File::create(&path)?;
                let c = Config::default();
                let v = toml::to_vec(&c).unwrap();
                f.write_all(&v)?;
            }
            // Base directory exists now.
            cf = File::open(&path)?;
        } else {
            return Err(Error::Directory)
        }
    }

    let mut buf = Vec::new();
    cf.read_to_end(&mut buf)?;

    toml::from_slice(&buf).map_err(Error::Toml)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_decode_test() {
        let cfg: Config = toml::from_str(r#"
            path = '/tmp/d/'

            [remotes]
                [remotes.test1]
                url = 'git@test1.example.com'

                [remotes.test2]
                url = 'https://test2.example.org'
        "#).unwrap();

        assert_eq!(cfg, Config {
            path: "/tmp/d".to_string().into(),
            remotes: hashmap!{
                "test1".to_string() => Remote { url: "git@test1.example.com".to_string() },
                "test2".to_string() => Remote { url: "https://test2.example.org".to_string() },
            },
        })
    }
}
