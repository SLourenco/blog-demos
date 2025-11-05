#[macro_use] extern crate rocket;

use rocket::serde::{Deserialize, Serialize};
use rocket::serde::json::Json;

#[derive(Serialize, Deserialize)]
struct Message<'r> {
    contents: &'r str,
}

#[post("/inventory/<id>/reserve")]
fn index(id: &str) -> String {
    format!("Reserved 1 item of {}", id)
}

#[get("/inventory/<id>/status")]
fn status(id: &str) -> String {
    format!("Quantity of product {}: 1", id)
}

#[put("/admin/inventory/<id>/refill", data = "<qty>")]
fn refill(id: &str, qty: Json<Message<'_>>) -> String {
    format!("Added {} items of {}", qty.contents, id)
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, status, refill])
}
