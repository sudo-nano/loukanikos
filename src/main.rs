use serde::{Serialize, Deserialize};
use serde_json::json;
use std::collections::BTreeMap;
use std::{
    env,
    fs::{self, File},
    io::prelude::*,
    io::{self, BufReader},
    process::Command,
};
use toml::toml;
use pcap::{Device, Capture};
use macaddr::{MacAddr6, MacAddr8};
use u4::U4;

enum MacPrefix {
    Small([U4; 9]),  // 4.5 byte prefix
    Medium([U4; 7]), // 3.5 byte prefix
    Large([U4; 6])  // 3 byte prefix
}

#[derive(Serialize, Deserialize)]
struct Company {
    #[serde(rename = "Exhibitor")]
    name: String,
    #[serde(rename = "Prefixes")]
    prefixes: Vec<String>,
}

fn main() {
    let devices = Device::list().expect("Could not get capture devices.");

    // Use default device unless an argument is specified
    let mut interface = Device::lookup().unwrap().expect("Unable to fetch default capture device");

    // If an interface is specified, make sure it's in the list of valid devices
    let arg2 = env::args().nth(1);
    if arg2 != None {
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
        println!("No interface specified. Capturing on default interface {}", interface_name);
    }

    dbg!(&interface);

    let prefix_db: Vec<Company> = Vec::new();
    import_toml(&prefix_db);

    capture_pcap(interface, prefix_db);
}

fn capture_pcap(interface: Device, db: Vec<Company>) {
    let mut capture = Capture::from_device(interface).unwrap()
        .promisc(true)
        .rfmon(true)
        .open().expect("Unable to open socket");
    while let Ok(packet) = capture.next_packet() {
        println!("Received packet! {:?}", packet);
    }
}

fn capture_tcpdump(interface: Device, db: Vec<Company>) {
    // TODO: Write function to build BPF filter from TOML files

    // TODO: Pass TCPdump the filter file
    let output = Command::new("sudo")
        .arg("tcpdump")
        .arg("-i")
        .arg(interface.name)
        .arg("-e")
        .spawn()
        .expect("Failed to start tcpdump");
}

fn json2toml () {
    let dir_path = "Companies/";
    let dir = fs::read_dir(dir_path).expect("Could not find directory");
    let mut i = 0;
    for path in dir {
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
        i += 1;
    }
}

// Load toml files into internal data
// TODO: After creating internal struct for the data, add a vector of them as the parameter for this function
fn import_toml (db: &Vec<Company>) {
    let dir_path = "Companies/tomls/";
    let dir = fs::read_dir(dir_path).expect("Could not find directory");
    for path in dir {
        let path_unwrapped = path.expect("Invalid file path");
        dbg!(path_unwrapped.path().display());

        let contents = fs::read_to_string(path_unwrapped.path())
            .expect("Could not open file");
        dbg!(&contents);

        let table = toml::Value::try_from(contents).expect("Could not convert TOML to table");
        dbg!(table);
        // TODO: Deserialize table
    }
}

// Create tcpdump filter from already imported files
// TODO: After creating internal struct for the data, add a vector of them as the parameter for this function
fn create_tcpdump_filter (db: Vec<Company>) {

}
