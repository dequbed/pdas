use std::path::PathBuf;

use serde::{Serialize, Deserialize};

use directories::ProjectDirs;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Config {
    #[serde(default)]
    pub database: DBConfig,
    #[serde(default)]
    pub backend: BackendConfig,
}

impl Config {
    pub fn to_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(self)
    }

    pub fn from_str<'de>(s: &'de str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DBConfig {
    pub basedir: PathBuf,
}
impl Default for DBConfig {
    fn default() -> Self {
        Self {
            basedir: ProjectDirs::from("org", "Paranoidlabs", "Librarian").unwrap().data_dir().to_path_buf(),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct BackendConfig {
    pub mechanism: Mech,
}

#[derive(Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Mech {
    SHA256E,
}
impl Default for Mech {
    fn default() -> Self { Mech::SHA256E }
}
