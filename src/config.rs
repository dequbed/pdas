use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct Config {
    path: PathBuf,

    remotes: HashMap<String, Remote>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
struct Remote {
    url: String,
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
