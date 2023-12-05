use std::{collections::HashMap, fs};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct ModsFile {
    pub game: String,
    pub mods: HashMap<String, Mod>,
}

#[derive(Deserialize)]
pub struct Mod {
    pub name: String,
    pub file_id: u32,
    pub file_name: String,
}

impl ModsFile {
    pub fn parse() -> ModsFile {
        let contents = fs::read_to_string("./mods.toml").expect("Could not find mods.toml");

        return toml::from_str(&contents).unwrap();
    }
}
