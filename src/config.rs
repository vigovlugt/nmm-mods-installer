use std::{collections::HashMap, fs};

use serde::Deserialize;

#[derive(Deserialize)]
pub struct ModsFile {
    pub game: String,
    pub mods: HashMap<String, Mod>,
}

#[derive(Deserialize)]
pub struct Mod {
    pub main_file: String,
    pub optional_files: Option<Vec<String>>,
    pub misc_files: Option<String>,
}

impl ModsFile {
    pub fn parse() -> ModsFile {
        let contents = fs::read_to_string("./mods.toml").expect("Could not find mods.toml");

        toml::from_str(&contents).unwrap()
    }
}

pub fn get_id(string: &str) -> u32 {
    return string.split('_').next().unwrap().parse::<u32>().unwrap();
}

pub fn get_name(string: &str) -> String {
    return string.split('_').skip(1).collect::<Vec<_>>().join("_");
}
