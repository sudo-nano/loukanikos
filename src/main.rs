use serde;
use serde_json::json;
use std::collections::BTreeMap;
use std::{
    env,
    fs::{self, File},
    io::prelude::*,
    io::{self, BufReader},
};
use toml::toml;

fn main() {
    let dir_path = "Companies/";
    let dir = fs::read_dir(dir_path).expect("Could not find directory");
    let mut i = 0;
    for f in dir {
        let f_unwrapped = f.expect("Invalid file path");
        print!("{}", f_unwrapped.path().display());
        let file = fs::File::open(f_unwrapped.path()).expect("Could not open file");
        let buf_reader = BufReader::new(file);
        let data =
            serde_json::from_reader::<BufReader<File>, Vec<BTreeMap<String, String>>>(buf_reader)
                .expect("Could not create string from input");
        dbg!(&data);
        let new_file_str = toml::to_string_pretty(&data).expect("Could not convert string to TOML");
        let new_file_bytes = new_file_str.as_bytes();
        let mut new_file = File::create(i.to_string() + ".toml").expect("Could not create file");
        let _ = new_file.write_all(new_file_bytes);
        i += 1;
    }
}
