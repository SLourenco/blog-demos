use rocket::serde::json::Json;
use rocket::{State, launch, post, routes, tokio::sync::Mutex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::format;
use std::sync::{Arc};
use uuid::{Uuid};
use reqwest::Error;

#[derive(Serialize, Deserialize)]
struct OrderRequest {
    address: String,
    price: f32,
    product: String
}

struct DB {
    data: HashMap<String, OrderRequest>,
}

type AppState = Arc<Mutex<DB>>;

#[post("/order", data = "<body>")]
async fn order(db: &State<AppState>, body: Json<OrderRequest>) -> String {
    let data = &mut db.lock().await.data;
    let id = Uuid::new_v4();
    data.insert(
        id.to_string(),
        OrderRequest {
            address: body.address.clone(),
            price: body.price,
            product: body.product.clone()
        },
    );

    reserve_product(body.product.clone()).await.expect("TODO: panic message");
    make_payment(body.price).await.expect("TODO : Panic message");
    schedule_delivery(body.address.clone()).await.expect("TODO: panic message");

    format!(
        "Order ID: {}, (Product: {}, Price: {}, Address: {})",
        id.to_string(),
        body.product,
        body.price,
        body.address
    )
}

async fn reserve_product(product: String) -> Result<usize, Error> {
    let client = reqwest::Client::new();
    let res = client.post(format!("http://localhost:8050/inventory/{}/reserve", product))
        .send()
        .await?;

    println!("body = {:?}", res.text().await);
    Ok(1)
}

async fn make_payment(_: f32) -> Result<usize, Error> {
    let client = reqwest::Client::new();
    let res = client.post("http://localhost:8060/payment")
        .body("{\"account\": 1,\"value\": 2000 }")
        .send()
        .await?;

    println!("body = {:?}", res.text().await);
    Ok(1)
}

async fn schedule_delivery(_: String) -> Result<usize, Error> {
    let client = reqwest::Client::new();
    let res = client.post("http://localhost:8070/delivery")
        .body("{\"address\": \"Rua da avenida, sitio do lugar\"}")
        .send()
        .await?;

    println!("body = {:?}", res.text().await);
    Ok(1)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .configure(rocket::Config::figment().merge(("port", 8090)))
        .manage(Arc::new(Mutex::new(DB {
            data: HashMap::<String, OrderRequest>::new(),
        })))
        .mount("/", routes![order])
}

trait Process {
    fn prepare(&self) -> bool;
    fn commit(&self);
}

struct Coordinator {
    processes: Vec<&'static dyn Process>,
}

impl Coordinator {
    fn process_order(&self) {
        let mut ok = true;
        for process in self.processes.iter() {
            ok =  ok && process.prepare();
        }

        if (ok) {
            for process in self.processes.iter() {
                process.commit()
            }
        }
    }
}