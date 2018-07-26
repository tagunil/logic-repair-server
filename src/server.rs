use std::sync::{Arc, Mutex};

use rocket;
use rocket_contrib::Json;

use super::System;
use super::Systems;

#[get("/")]
fn index(
    state: rocket::State<Arc<Mutex<Systems>>>
) -> Json<Vec<usize>> {
    let systems = state.lock().unwrap();
    let mut indexes: Vec<usize> = systems.keys().map(|reference| *reference).collect();
    indexes.sort_unstable();
    Json(indexes)
}

#[get("/<index>")]
fn get_system(
    index: usize,
    state: rocket::State<Arc<Mutex<Systems>>>,
) -> Option<Json<System>> {
    let systems = state.lock().unwrap();
    match systems.get(&index) {
        Some(server_system) => Some(Json(*server_system)),
        None => None,
    }
}

#[post("/<index>", data = "<client_system>")]
fn set_system(
    index: usize,
    client_system: Json<System>,
    state: rocket::State<Arc<Mutex<Systems>>>,
) -> Option<Json<System>> {
    let mut systems = state.lock().unwrap();
    match systems.get_mut(&index) {
        Some(server_system) => {
            if server_system.programmed != client_system.programmed {
                server_system.programmed = client_system.programmed;
                if server_system.programmed == 0x0000 {
                    server_system.corrected = Some(0x0000);
                } else {
                    server_system.corrected = Some(0xffff);
                }
            }
            Some(Json(*server_system))
        },
        None => None,
    }
}

#[catch(400)]
fn bad_request() -> Json<()> {
    Json(())
}

#[catch(404)]
fn not_found() -> Json<()> {
    Json(())
}

#[catch(500)]
fn internal_error() -> Json<()> {
    Json(())
}

pub(crate) fn run(shared_systems: Arc<Mutex<Systems>>) {
    rocket::ignite()
          .mount("/system", routes![index, get_system, set_system])
          .manage(shared_systems)
          .catch(catchers![bad_request, not_found, internal_error])
          .launch();
}
