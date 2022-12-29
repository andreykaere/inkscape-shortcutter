use std::collections::HashMap;

lazy_static::lazy_static! {
    pub static ref CONFIG: Config = Config::new();
}

pub struct Config {
    pub motions: HashMap<char, i32>,
}

// pub fn solid() {}

impl Config {
    pub fn new() -> Self {
        let motions = HashMap::from([('s', 1)]);
        Self { motions }
    }
}
