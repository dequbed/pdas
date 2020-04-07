use serde::Deserialize;
use config::{Config, ConfigError, File, FileSourceFile, Environment};
use dirs;
use std::fs::DirBuilder;
use std::env;

use std::path::{Path, PathBuf};

#[derive(Debug,Deserialize)]
/// PDAS application settings
///
/// Settings are either compiled in default values (from `impl Default`), values set in the
/// configuration file, environment-variable overrides or command line arguments in increasing
/// order.
// TODO: Make a proc macro for pulling env variables and other merging?
pub struct Settings {
    pub databasepath: PathBuf,
    #[serde(default)]
    pub loglevel: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            databasepath: PathBuf::from(""),
            loglevel: slog::Level::Error.as_usize(),
        }
    }
}

impl Settings {
    pub fn new<P: AsRef<Path>>(log: &slog::Logger, path: Option<P>) -> Result<Self, ConfigError> {
        let mut s = Config::new();

        // Read a configuration file
        if let Some(path) = path {
            debug!(log, "Reading config file '{}'", path.as_ref().display());

            let configfile: File<FileSourceFile> = path.as_ref().into();
            s.merge(configfile.required(false))?;
        } else {
            if let Some(dir) = dirs::config_dir() {
                let configpath: PathBuf = dir.join("pdas/config.toml");

                debug!(log, "Reading config file '{}'", configpath.display());

                let configfile: File<FileSourceFile> = configpath.into();
                s.merge(configfile.required(false))?;
            }
        };

        // Use environment variables
        s.merge(Environment::with_prefix("pdas"))?;

        s.try_into()
    }

    pub fn set_loglevel(&mut self, level: slog::Level) {
        self.loglevel = level.as_usize();
    }
}
