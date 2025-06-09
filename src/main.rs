use macaddr::{MacAddr6, MacAddr8};
use pcap::{Capture, Device};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::io::{BufWriter, ErrorKind};
use std::{
    env,
    fs::{self, File},
    io::prelude::*,
    io::BufReader,
    process::{Command, Stdio},
};
use u4::U4;
use std::path::Path;

#[derive(Debug)]
enum MacPrefix {
    Small([U4; 9]),  // 4.5 byte prefix
    Medium([U4; 7]), // 3.5 byte prefix
    Large([U4; 6]),  // 3 byte prefix
}

impl MacPrefix {
    fn from_str(string: &str) {
        let small = Regex::new(r"([0-9a-f]{2}:){4}[0-9a-f]").unwrap();
        let medium = Regex::new(r"([0-9a-f]{2}:){3}[0-9a-f]").unwrap();
        let large = Regex::new(r"([0-9a-f]{2}:){3}").unwrap();

        let match_small = small.find(&string);
        let match_medium = medium.find(&string);
        let match_large = large.find(&string);

        // TODO: Check options returned by regex finds. Reject matches at nonzero
        // offsets. In cases with multiple matches, keep largest match.
    }
}

#[derive(Deserialize)]
struct Category {
    name: String,
    companies: Vec<Company>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Company {
    #[serde(rename = "Exhibitor")]
    name: String,
    #[serde(rename = "Prefixes")]
    prefixes: Option<Vec<String>>,
}

fn main() {
    let devices = Device::list().expect("Could not get capture devices.");

    // Use default device unless an argument is specified
    let mut interface = Device::lookup()
        .unwrap()
        .expect("Unable to fetch default capture device");

    // If an interface is specified, make sure it's in the list of valid devices
    let arg2 = env::args().nth(1);
    if arg2.is_some() {
        let interface_name = arg2.unwrap();
        let mut name_valid = false;
        for device in &devices {
            if device.name == interface_name {
                interface = device.clone();
                name_valid = true;
                break;
            }
        }
        if !name_valid {
            panic!("Not a valid capture interface.");
        }
    } else {
        let interface_name = interface.name.as_str();
        println!(
            "No interface specified. Capturing on default interface {}",
            interface_name
        );
    }

    dbg!(&interface);

    // Initialize prefix database
    let mut prefix_db: Vec<Company> = Vec::new();

    // Import toml directory
    let dir = "./Companies/tomls/";
    let _ = import_toml_dir(dir, &mut prefix_db);

    // TODO: Prompt user for whether to use direct pcap capture or tcpdump capture
    let capture_result = capture_tcpdump(interface, &prefix_db);
    match capture_result {
        Ok(something) => something,
        Err(e) => panic!("tcpdump capture failed"),
    }
    //capture_pcap(interface, prefix_db);
}

fn capture_pcap(interface: Device, db: &[Company]) {
    let mut capture = Capture::from_device(interface)
        .unwrap()
        .promisc(true)
        .rfmon(true)
        .open()
        .expect("Unable to open socket");
    while let Ok(packet) = capture.next_packet() {
        // TODO: write function for filtering received pcap format packets
        println!("Received packet! {:?}", packet);
    }
}

/// Initiate capture using tcpdump
fn capture_tcpdump (interface: Device, db: &Vec<Company>) -> Result<(), std::io::Error> {
    let output = Command::new("sudo")
        .arg("tcpdump")
        .arg("-i")
        .arg(interface.name)
        .arg("-e")
        .stdout(Stdio::piped())
        .spawn()?
        .stdout
        .ok_or_else(|| std::io::Error::new(ErrorKind::Other, "Could not capture standard output"))?;

    // Initialize regex for extracting MAC addresses
    let mac_extractor = Regex::new("(?:[0-9a-f]{2}:){5}[0-9a-f]{2}").unwrap();

    // Initialize buffered reader for the output
    let reader = BufReader::new(output);

    for line in reader.lines() {
        dbg!(&line);
        let line_unwrapped = line.unwrap();
        let line_str = line_unwrapped.as_str();
        let extracted_macs: Vec<&str> = mac_extractor.find_iter(line_str).map(|m| m.as_str()).collect();
        dbg!(&extracted_macs);

        // TODO: Differentiate between matching the first of the extracted MAC
        // (sender) and second of extracted MACs (recipient)
        for mac in extracted_macs {
            let company = check_prefix(&mac, db);
            if company.is_some() {
                // TODO: If a company has multiple prefixes, track which one it matches.
                // Doing so may be useful for statistics.
                println!("[ALERT] Address {} matches company {}", mac, company.unwrap().name);
            }
        }
    }

    Ok(())
}

/// Check a MAC address against the prefix database
fn check_prefix<'a>(mac: &str, db: &'a Vec<Company>) -> Option<&'a Company> {
    for company in db {
        if company.prefixes.is_some() {
            for prefix in company.prefixes.as_ref().unwrap() {
                let results = mac.find(prefix.as_str());
                if results? == 0 {
                    return Some(&company);
                }
            }
        }
    }
    None
}

fn json2toml() {
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
fn import_toml(path: &str, db: &mut Vec<Company>) -> Result<(), toml::de::Error> {
    // Validate path
    let file = fs::read_to_string(path);
    if let Ok(toml_file) = file {
        let slice = toml_file.as_str();
        let categories: HashMap<String, Vec<Company>> = toml::from_str(slice)?;
        for (_, companies) in categories.iter() {
            for company in companies {
                if company.prefixes.is_some() {
                    db.push(company.clone());
                }
            }
        }
    }
    Ok(())
}

fn import_toml_dir(dir_str: &str, db: &mut Vec<Company>) -> Result<(), std::io::Error> {
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
