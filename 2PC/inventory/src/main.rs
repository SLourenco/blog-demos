#[macro_use]
extern crate rocket;

use rocket::State;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct Message {
    quantity: usize,
}

#[derive(Serialize, Deserialize)]
struct Response {
    id: String,
    error: Option<String>,
}

struct DB {
    inventory: HashMap<String, usize>,
    reservations: HashMap<String, usize>,
}

type AppState = Arc<Mutex<DB>>;

#[post("/reserve/<id>", data = "<qty>")]
fn reserve(db: &State<AppState>, id: &str, qty: Json<Message>) -> Json<Response> {
    let db = &mut db.lock().unwrap();
    let reservation_id = Uuid::new_v4();

    let existing_qty = *db.inventory.get(id).or(Some(&0usize)).unwrap();
    if existing_qty <= qty.quantity {
        Json(Response {
            id: reservation_id.to_string(),
            error: Some(format!(
                "Not enough quantity of item {} (current qty: {})",
                id, existing_qty
            )),
        })
    } else {
        db.reservations.insert(String::from(id), qty.quantity);
        db.inventory.insert(reservation_id.to_string(), existing_qty - qty.quantity);
        Json(Response {
            id: reservation_id.to_string(),
            error: None,
        })
    }
}

#[post("/commit/<id>")]
fn commit(db: &State<AppState>, id: &str) {
    let reservations = &mut db.lock().unwrap().reservations;
    reservations.remove(id);
}

#[put("/refill/<id>", data = "<qty>")]
fn refill(db: &State<AppState>, id: &str, qty: Json<Message>) -> String {
    let data = &mut db.lock().unwrap().inventory;
    let inventory = data.get(id).or(Some(&0usize)).unwrap();
    data.insert(String::from(id), *inventory + qty.quantity);
    format!("Added {} items of {}", qty.quantity, id)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .configure(rocket::Config::figment().merge(("port", 7050)))
        .manage(Arc::new(Mutex::new(DB {
            inventory: HashMap::<String, usize>::new(),
            reservations: HashMap::<String, usize>::new(),
        })))
        .mount("/", routes![reserve, commit, refill])
}
