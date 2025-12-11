use rocket::serde::json::Json;
use rocket::{State, launch, post, routes};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct DeliveryRequest {
    address: String
}

#[derive(Serialize, Deserialize, Clone)]
struct DeliveryResponse {
    id: String,
    address: String,
    date: String
}

struct DB {
    data: HashMap<String, DeliveryResponse>,
    confirmed: Vec<String>
}

type AppState = Arc<Mutex<DB>>;

#[post("/schedule", data = "<body>")]
fn delivery(db: &State<AppState>, body: Json<DeliveryRequest>) -> Json<DeliveryResponse> {
    let data = &mut db.lock().unwrap().data;
    let id = Uuid::new_v4();
    let eta = chrono::offset::Local::now()
        .checked_add_days(chrono::Days::new(5))
        .unwrap();
    let response = DeliveryResponse {
        address: body.address.clone(),
        date: eta.to_rfc2822(),
        id: id.to_string()
    };
    data.insert(
        id.to_string(),
        response.clone(),
    );
    Json(response)
}

#[post("/confirm/<id>")]
fn confirm(db: &State<AppState>, id: &str) -> Json<DeliveryResponse> {
    let db = &mut db.lock().unwrap();
    let schedule = db.data.get(id).expect("unexpected");
    let eta = chrono::offset::Local::now()
        .checked_add_days(chrono::Days::new(5))
        .unwrap();

    let response = DeliveryResponse {
        address: schedule.address.clone(),
        date: eta.to_rfc2822(),
        id: id.to_string()
    };
    db.confirmed.push(id.to_string());
    db.data.insert(
        id.to_string(),
        response.clone(),
    );
    Json(response)
}

#[post("/rollback/<id>")]
fn rollback(db: &State<AppState>, id: &str) {
    let db = &mut db.lock().unwrap();
    db.data.remove(id);
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .configure(rocket::Config::figment().merge(("port", 7070)))
        .manage(Arc::new(Mutex::new(DB {
            data: HashMap::<String, DeliveryResponse>::new(),
            confirmed: Vec::new()
        })))
        .mount("/", routes![delivery, confirm, rollback])
}
