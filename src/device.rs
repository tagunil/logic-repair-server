use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::io::{self, BufRead, BufReader, Write, LineWriter};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use serialport::{self, SerialPortType};

use hex;

use super::System;
use super::Systems;

const USB_VID: u16 = 0x0483;
const USB_PID: u16 = 0x5740;

fn find_port() -> Option<String> {
    for port_info in serialport::available_ports().unwrap() {
        if let SerialPortType::UsbPort(usb_info) = port_info.port_type {
            if (usb_info.vid == USB_VID) && (usb_info.pid == USB_PID) {
                return Some(port_info.port_name);
            }
        }
    }

    None
}

fn read_systems<T: BufRead, U: Write>(
    reader: &mut T,
    writer: &mut U,
) -> Result<Systems, io::Error> {
    let request = "READ 50 32\r\n";
    writer.write_all(request.as_bytes())?;

    let mut response: String = String::new();
    reader.read_line(&mut response)?;

    let mut words = response.split_whitespace();

    match words.next() {
        Some("DATA") => Ok(()),
        Some("ERROR") => Err(io::Error::new(io::ErrorKind::InvalidData, "Device error")),
        Some(_) => Err(io::Error::new(io::ErrorKind::InvalidData, "Unknown response")),
        None => Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid response")),
    }?;

    let data = match words.next() {
        Some(string) if string.len() == 64 => match hex::decode(string) {
            Ok(bytes) => Ok(bytes),
            Err(_) => Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid data")),
        },
        Some(_) => Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid data length")),
        None => Err(io::Error::new(io::ErrorKind::InvalidData, "Missing data")),
    }?;

    let time_now = SystemTime::now();
    let timestamp = time_now.duration_since(UNIX_EPOCH).unwrap().as_secs();

    let mut systems: Systems = HashMap::new();

    for index in 0..8 {
        let base = index * 4;

        let high = data[base + 1] as u16;
        let low = data[base + 0] as u16;
        let programmed = (high << 8) + low;

        let high = data[base + 3] as u16;
        let low = data[base + 2] as u16;
        let corrected = (high << 8) + low;

        systems.insert(index, System {
            programmed: programmed,
            corrected: Some(corrected),
            timestamp: Some(timestamp),
        });
    }

    Ok(systems)
}

fn write_systems<T: BufRead, U: Write>(
    systems: &Systems,
    reader: &mut T,
    writer: &mut U,
) -> Result<(), io::Error> {
    if systems.is_empty() {
        return Ok(());
    }

    let mut data: Vec<u8> = Vec::new();

    for (&index, system) in systems.iter() {
        data.push(0xc5);
        data.push(index as u8);
        data.push(system.programmed as u8);
        data.push((system.programmed >> 8) as u8);
    }

    let request = format!("WRITE 50 {} {}\r\n", data.len(), hex::encode(data));
    writer.write_all(request.as_bytes())?;

    let mut response: String = String::new();
    reader.read_line(&mut response)?;

    let mut words = response.split_whitespace();

    match words.next() {
        Some("OK") => Ok(()),
        Some("ERROR") => Err(io::Error::new(io::ErrorKind::InvalidData, "Device error")),
        Some(_) => Err(io::Error::new(io::ErrorKind::InvalidData, "Unknown response")),
        None => Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid response")),
    }
}

fn try_sync(
    port_name: &str,
    shared_systems: &Arc<Mutex<Systems>>,
) -> Result<(), io::Error> {
    let mut port = serialport::open(port_name)?;
    port.set_timeout(Duration::from_millis(500))?;

    let mut reader = BufReader::new(port.try_clone().unwrap());
    let mut writer = LineWriter::new(port.try_clone().unwrap());

    let mut device_systems = read_systems(&mut reader, &mut writer)?;
    let mut changed_systems: Systems = HashMap::new();

    {
        let mut server_systems = shared_systems.lock().unwrap();
        for (&index, device_system) in device_systems.iter_mut() {
            let server_system = server_systems.entry(index).or_insert(*device_system);
            if device_system.programmed != server_system.programmed {
                changed_systems.insert(index, *server_system);
            } else {
                server_system.corrected = device_system.corrected;
            }
            server_system.timestamp = device_system.timestamp;
        }
    }

    if !changed_systems.is_empty() {
        write_systems(&changed_systems, &mut reader, &mut writer)?;
    }

    Ok(())
}

pub(crate) fn run(shared_systems: Arc<Mutex<Systems>>) {
    loop {
        if let Some(port_name) = find_port() {
            if let Err(_error) = try_sync(&port_name, &shared_systems) {
                //TODO: do something
            }
        }

        thread::sleep(Duration::from_millis(1000));
    }
}
