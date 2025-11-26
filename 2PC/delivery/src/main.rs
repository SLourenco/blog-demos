use rocket::serde::json::Json;
use rocket::{State, launch, post, routes};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct DeliveryRequest {
    address: String,
    date: Option<String>,
}

struct DB {
    data: HashMap<String, DeliveryRequest>,
}

type AppState = Arc<Mutex<DB>>;

#[post("/delivery", data = "<body>")]
fn delivery(db: &State<AppState>, body: Json<DeliveryRequest>) -> String {
    let data = &mut db.lock().unwrap().data;
    let id = Uuid::new_v4();
    let eta = chrono::offset::Local::now()
        .checked_add_days(chrono::Days::new(5))
        .unwrap();
    data.insert(
        id.to_string(),
        DeliveryRequest {
            address: body.address.clone(),
            date: Some(eta.to_rfc2822()),
        },
    );
    format!(
        "Delivery ID: {}, ETA: {}, (Address: {})",
        id.to_string(),
        eta.to_rfc2822(),
        body.address
    )
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .configure(rocket::Config::figment().merge(("port", 8070)))
        .manage(Arc::new(Mutex::new(DB {
            data: HashMap::<String, DeliveryRequest>::new(),
        })))
        .mount("/", routes![delivery])
}
