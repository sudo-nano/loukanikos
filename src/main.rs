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
use pcap::{Device, Capture};

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

    capture_pcap(interface);
}

fn capture_pcap(interface: Device) {
    let mut capture = Capture::from_device(interface).unwrap()
        .promisc(true)
        .rfmon(true)
        .open().unwrap();
    while let Ok(packet) = capture.next_packet() {
        println!("Received packet! {:?}", packet);
    }
}

fn capture_tcpdump(interface: Device) {
    // TODO: Write this
}

fn json2toml () {
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
