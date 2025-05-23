use macaddr::{MacAddr6, MacAddr8};
use pcap::{Capture, Device};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::io::BufWriter;
use std::{
    env,
    fs::{self, File},
    io::prelude::*,
    io::BufReader,
    process::Command,
};
use u4::U4;

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

    let mut prefix_db: Vec<Company> = Vec::new();
    // TODO: change this to the tomls directory and then iterate through the files in it
    let path = "./Companies/tomls/unmanned_vehicles_robotics.toml";
    let _ = import_toml(path, &mut prefix_db);

    // TODO: Prompt user for whether to use direct pcap capture or tcpdump capture
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

fn capture_tcpdump(interface: Device, db: &[Company]) {
    // TODO: Write function to build BPF filter from TOML files

    // TODO: Pass TCPdump the filter file
    let output = Command::new("sudo")
        .arg("tcpdump")
        .arg("-i")
        .arg(interface.name)
        .arg("-e")
        .spawn()
        .expect("Failed to start tcpdump")
        .wait()
        .expect("Child process failed");
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

// Create tcpdump filter from already imported files
fn create_tcpdump_filter(db: &[Company]) -> Result<(), std::io::Error>{
    let filter_file =
        File::create("filterfile.txt").expect("Could not create filterfile.txt for TCPDump");
    let mut buf_writer = BufWriter::new(filter_file);
    for company in db.iter() {
        if company.prefixes.is_none() {
            continue;
        }
        let prefixes = company.prefixes.as_ref().unwrap();
        if prefixes.is_empty() {
            continue;
        }
        for prefix in prefixes.iter() {
            // Unfortunately BPF prefix matching sucks because of this
            // https://stackoverflow.com/questions/55687405/why-does-bpf-allow-ether02-and-ether04-but-not-ether03
            let prefix_len = match prefix.len() {
                8 => 3,  // Probably need to change this so that BPF is happy
                9 => 4,  // Actually 3.5 bytes
                13 => 5, // Actually 4.5 bytes
                _ => panic!("Invalid MAC prefix length")
            };
            // TODO: Verify that this actually matches 3.5 and 4.5 byte prefixes properly
            let line = format!("ether [6:{}] == {} or", prefix_len, prefix);
            buf_writer.write_all(line.as_bytes())?;
        }
    }

    buf_writer.flush()?;
    Ok(())
}
