
use etherparse::SlicedPacket;
use pcap::{Capture, Device};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
use std::{
    env,
    io::prelude::*,
    io::BufReader,
    process::{Command, Stdio},
};
use u4::U4;

use hex_string::u8_to_hex_string;

mod data_conversion;
use data_conversion::Company;

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



fn main() {
    println!("");

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

    // Check for flag to use pcap
    // TODO: Use a real argument parsing library
    let mut use_pcap = false;
    let arg3 = env::args().nth(2);
    if arg3.is_some() {
        if arg3.unwrap() == "--use-pcap" {
            use_pcap = true;
        }
    }

    dbg!(&interface);

    // Initialize prefix database
    let mut prefix_db: Vec<Company> = Vec::new();

    // Import toml directory
    let dir = "./Companies/tomls/";
    let _ = data_conversion::import_toml_dir(dir, &mut prefix_db);

    // DEBUG: Test database matching
    let test_result_0 = check_prefix("00:25:DF:ab:cd:ef", &prefix_db);
    if test_result_0.is_none() {
        println!("[DEBUG] Test match of 00:25:DF failed. Exiting.");
        panic!();
    }

    let test_result_1 = check_prefix("00:25:df:ab:cd:ef", &prefix_db);
    if test_result_1.is_none() {
        println!("[DEBUG] Test match of 00:25:df (lowercase) failed. Exiting.");
        panic!();
    }

    // Capture using either pcap or tcpdump depending on flag
    if use_pcap {
        println!("Capturing with pcap.");
        capture_pcap(interface, &prefix_db);
    } else {
        println!("Capturing with tcpdump.");
        let capture_result = capture_tcpdump(interface, &prefix_db);
        match capture_result {
            Ok(something) => something,
            Err(e) => panic!("tcpdump capture failed"),
        }
    }
}

/// Initiate capture using the pcap library
fn capture_pcap(interface: Device, db: &Vec<Company>) {
    let mut capture = Capture::from_device(interface)
        .unwrap()
        .promisc(true)
        .rfmon(true)
        .open()
        .expect("Unable to open socket");
    while let Ok(packet) = capture.next_packet() {
        // TODO: parse packets with either etherparse or pnet
        println!("Received packet!");
        match SlicedPacket::from_ethernet(&packet) {
            Err(value) => println!("Err {:?}", value),
            Ok(value) => {
                // TODO: stuff all of this messy conversion into a function that takes
                // a SlicedPacket in and outputs the source and destination MAC

                // Unwrapping here produces a LinkSlice
                let slice = value.link.unwrap();

                // Unwrapping here produces an Ethernet2Header
                let ether2header = slice.to_header()
                    .unwrap()
                    .ethernet2()
                    .unwrap();
                println!("source array: {:?}", ether2header.source);
                let source_string = mac_u8_to_string(ether2header.source);
                println!("source: {}", source_string);
                println!();
                let company = check_prefix(source_string.as_str(), db);
                if company.is_some() {
                    // TODO: If a company has multiple prefixes, track which one it matches.
                    // Doing so may be useful for statistics.
                    // TODO: Implement optional matching of destination address for extended
                    // detection
                    println!("[ALERT] Address {} matches company {}", source_string, company.unwrap().name);
                }
            }
        }
    }
}

// Convert an array of 6 u8s into a MAC address String with colon separated octets
fn mac_u8_to_string(u8_array: [u8;6]) -> String{
    let mut mac_str = String::new();
    for octet in u8_array {
        let octet_str = u8_to_hex_string(&octet);
        mac_str.push(octet_str[0]);
        mac_str.push(octet_str[1]);
    }
    return mac_str
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
        let line_unwrapped = line.unwrap();
        let line_str = line_unwrapped.as_str();
        let extracted_macs: Vec<&str> = mac_extractor.find_iter(line_str).map(|m| m.as_str()).collect();

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
    let mac_lower = &mac.to_lowercase();
    for company in db {
        if company.prefixes.is_some() {
            for prefix in company.prefixes.as_ref().unwrap() {
                let results = &mac_lower.find(prefix);
                if results.is_some() {
                    if results.unwrap() == 0 {
                        return Some(&company);
                    }
                }
            }
        }
    }
    //println!("[DEBUG] MAC {} does not match any database entries.", mac);
    None
}
