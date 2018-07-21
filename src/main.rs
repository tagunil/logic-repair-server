#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate serde_derive;
extern crate serialport;
extern crate rocket;
extern crate rocket_contrib;
extern crate hex;

mod device;
mod server;

use std::thread;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Copy)]
struct System {
    programmed: u16,
    corrected: Option<u16>,
    timestamp: Option<u64>,
}

type Systems = HashMap<usize, System>;

fn main() {
    let shared_systems: Arc<Mutex<Systems>> = Arc::new(Mutex::new(HashMap::new()));

    let device_shared_systems = Arc::clone(&shared_systems);
    let device_thread = thread::spawn(move || {
        device::run(device_shared_systems);
    });

    let server_shared_systems = Arc::clone(&shared_systems);
    let server_thread = thread::spawn(move || {
        server::run(server_shared_systems);
    });

    server_thread.join().unwrap();
    device_thread.join().unwrap();
}
