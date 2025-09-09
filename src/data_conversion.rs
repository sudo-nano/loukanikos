// Data cleaning and management functions for loukanikos
use std::{
    env,
    fs::{self, File},
    io::prelude::*,
    io::BufReader,
    process::{Command, Stdio},
};
use std::path::Path;
use crate::Deserialize;
use crate::Serialize;
use std::collections::{HashMap, BTreeMap};

#[derive(Deserialize)]
pub struct Category {
    pub name: String,
    pub companies: Vec<Company>,
}

/// Struct for storing the name and prefixes of a company
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Company {
    #[serde(rename = "Exhibitor")]
    pub name: String,
    #[serde(rename = "Prefixes")]
    pub prefixes: Option<Vec<String>>,
}

pub fn json2toml() {
    let dir_path = "Companies/";
    let dir = fs::read_dir(dir_path).expect("Could not find directory");
    for (i, path) in dir.enumerate() {
        let path_unwrapped = path.expect("Invalid file path");
        print!("{}", path_unwrapped.path().display());
        let file = fs::File::open(path_unwrapped.path()).expect("Could not open file");
        let buf_reader = BufReader::new(file);
        let data =
            serde_json::from_reader::<BufReader<File>, Vec<BTreeMap<String, String>>>(buf_reader)
                .expect("Could not create string from input");
        dbg!(&data);
        let new_file_str = toml::to_string_pretty(&data).expect("Could not convert string to TOML");
        let new_file_bytes = new_file_str.as_bytes();
        let mut new_file = File::create(i.to_string() + ".toml").expect("Could not create file");
        let _ = new_file.write_all(new_file_bytes);
    }
}

// Load single toml file into internal data
pub fn import_toml(path: &str, db: &mut Vec<Company>) -> Result<(), toml::de::Error> {
    // Validate path
    let file = fs::read_to_string(path);
    if let Ok(toml_file) = file {
        let slice = toml_file.as_str();
        let categories: HashMap<String, Vec<Company>> = toml::from_str(slice)?;
        for (_, companies) in categories.iter() {
            for company in companies {
                if company.prefixes.is_some() {
                    let mut clone = company.clone();
                    let mut clone_prefixes = clone.prefixes.unwrap();
                    for i in 0..clone_prefixes.len() {
                        clone_prefixes[i] = clone_prefixes[i].to_ascii_lowercase();
                    }
                    clone.prefixes = Some(clone_prefixes);
                    db.push(clone);
                }
            }
        }
    }
    Ok(())
}

pub fn import_toml_dir(dir_str: &str, db: &mut Vec<Company>) -> Result<(), std::io::Error> {
    let dir = Path::new(dir_str);
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                // TODO: Implement proper error handling here. This is improper
                // error handling because I wanted to get this working before a protest.
                let import_result = import_toml(path.to_str().unwrap(), db);
                if import_result.is_err() {
                    panic!("Failed to import toml file");
                }
            }
        }
    }
    Ok(())
}
