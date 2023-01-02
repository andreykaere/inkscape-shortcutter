use std::collections::HashMap;

lazy_static::lazy_static! {
    pub static ref CONFIG: Config = Config::new();
}

pub struct Config {
    pub motions: Vec<&'static str>,
}

impl Config {
    pub fn new() -> Self {
        let motions = vec!["s", "t"];

        Self { motions }
    }

    pub fn execute(&self, key: &str) {
        match key {
            "s" => {}
            "t" => {}
            _ => {}
        }
    }
}
