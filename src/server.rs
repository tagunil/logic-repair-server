use std::sync::{Arc, Mutex};

use rocket;
use rocket_contrib::Json;

use super::System;
use super::Systems;

#[get("/", format = "application/json")]
fn index(
    state: rocket::State<Arc<Mutex<Systems>>>
) -> Json<Vec<usize>> {
    let systems = state.lock().unwrap();
    Json((0..systems.len()).collect())
}

#[get("/<id>", format = "application/json")]
fn get_system(
    id: usize,
    state: rocket::State<Arc<Mutex<Systems>>>,
) -> Option<Json<System>> {
    let systems = state.lock().unwrap();
    if id < systems.len() {
        Some(Json(systems[id]))
    } else {
        None
    }
}

#[post("/<id>", format = "application/json", data = "<system>")]
fn set_system(
    id: usize,
    system: Json<System>,
    state: rocket::State<Arc<Mutex<Systems>>>,
) -> Option<Json<System>> {
    let mut systems = state.lock().unwrap();
    if id < systems.len() {
        systems[id].programmed = system.programmed;
        Some(Json(systems[id]))
    } else {
        None
    }
}

pub(crate) fn run(shared_systems: Arc<Mutex<Systems>>) {
    rocket::ignite()
          .mount("/system", routes![index, get_system, set_system])
          .manage(shared_systems)
          .launch();
}
