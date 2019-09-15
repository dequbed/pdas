use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Config {
    db: DBConfig
}

struct DBConfig {
    port: u16,
}
