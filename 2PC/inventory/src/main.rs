#[macro_use] extern crate rocket;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rocket::serde::{Deserialize, Serialize};
use rocket::serde::json::Json;
use rocket::State;

#[derive(Serialize, Deserialize)]
struct Message {
    contents: usize,
}

struct DB {
    data: HashMap<String, usize>
}

type AppState = Arc<Mutex<DB>>;

#[post("/inventory/<id>/reserve")]
fn reserve(db: &State<AppState>, id: &str) -> String {
    let data = &mut db.lock().unwrap().data;
    let qty = *data.get(id).or(Some(&0usize)).unwrap();
    if qty <= 0usize {
        format!("Not enough quantity of item {}", id)
    } else {
        data.insert(String::from(id), qty - 1);
        format!("Reserved 1 item of {}", id)
    }
}

#[get("/inventory/<id>/status")]
fn status(db: &State<AppState>, id: &str) -> String {
    let data = &mut db.lock().unwrap().data;
    format!("Quantity of product {}: {}", id, data.get(id).or(Some(&0usize)).unwrap())
}

#[put("/admin/inventory/<id>/refill", data = "<qty>")]
fn refill(db: &State<AppState>, id: &str, qty: Json<Message>) -> String {
    let data = &mut db.lock().unwrap().data;
    let inventory = data.get(id).or(Some(&0usize)).unwrap();
    data.insert(String::from(id), *inventory + qty.contents);
    format!("Added {} items of {}", qty.contents, id)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .configure(rocket::Config::figment().merge(("port", 8050)))
        .manage(Arc::new(Mutex::new(DB {data: HashMap::<String, usize>::new()})))
        .mount("/", routes![reserve, status, refill])
}
