#[macro_use] extern crate rocket;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rocket::serde::{Deserialize, Serialize};
use rocket::serde::json::Json;
use rocket::State;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct PaymentRequest {
    account: usize,
    value: usize
}

struct DB {
    data: HashMap<String, PaymentRequest>
}

type AppState = Arc<Mutex<DB>>;

#[post("/payment", data = "<body>")]
fn payment(db: &State<AppState>, body: Json<PaymentRequest>) -> String {
    let data = &mut db.lock().unwrap().data;
    let id = Uuid::new_v4();
    // This could be a call to some 3rd party provider, to charge the value
    data.insert(id.to_string(), PaymentRequest{account: body.account, value: body.value});
    format!("Payment ID: {}, (Acc: {}, Value: {})", id.to_string(), body.account, body.value)
}

#[post("/payment/<id>/reversal")]
fn reversal(db: &State<AppState>, id: String) -> String {
    let data = &mut db.lock().unwrap().data;
    let payment = data.remove(&id);
    match payment {
        Some(p) =>
            format!(
                "Reversed payment ID: {}, (Acc: {}, Value: {})",
                id.to_string(),
                p.account,
                p.value
            ),
        None => format!("Payment {} not registered. Command ignored!", id)
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .configure(rocket::Config::figment().merge(("port", 8060)))
        .manage(Arc::new(Mutex::new(DB {data: HashMap::<String, PaymentRequest>::new()})))
        .mount("/", routes![payment, reversal])
}
