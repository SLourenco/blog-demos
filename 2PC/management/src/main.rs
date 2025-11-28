mod processes;

use rocket::serde::json::Json;
use rocket::{State, launch, post, routes, tokio::sync::Mutex};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Error;
use std::sync::Arc;
use uuid::Uuid;
use async_trait::async_trait;

#[derive(Serialize, Deserialize, Clone)]
struct OrderRequest {
    address: String,
    price: f32,
    product: String,
    quantity: usize
}

#[async_trait]
trait Process: Send + Sync {
    async fn prepare(&self, request: OrderRequest) -> Result<String, Error>;
    async fn commit(&self, id: String);
    async fn rollback(&self, id: String);
}

#[derive(Default)]
struct Coordinator {
    processes: Vec<Box<dyn Process>>,
    data: Arc<Mutex<HashMap<String, OrderRequest>>>,
}

impl Coordinator {
    async fn process_order(&self, request: OrderRequest) {
        let mut ok = true;
        let mut ids = Vec::new();
        for process in self.processes.iter() {
            let r = process.prepare(request.clone()).await;
            ok = ok && r.is_ok();
            if r.is_ok() {
                let s = r.ok().unwrap();
                print!("{}", s.clone());
                ids.push((process, s.clone()))
            }
        }

        if (ok) {
            for (process, id) in ids {
                process.commit(id).await
            }
        } else {
            for (process, id) in ids {
                process.rollback(id).await
            }
        }
    }
}

#[post("/order", data = "<body>")]
async fn order(coord: &State<Coordinator>, body: Json<OrderRequest>) -> String {
    let data = &mut coord.data.lock().await;
    let id = Uuid::new_v4();
    data.insert(
        id.to_string(),
        OrderRequest {
            address: body.address.clone(),
            price: body.price,
            product: body.product.clone(),
            quantity: body.quantity,
        },
    );

    coord.process_order(OrderRequest {
        address: body.address.clone(),
        price: body.price,
        product: body.product.clone(),
        quantity: body.quantity,
    }).await;

    format!(
        "Order ID: {}, (Product: {}, Price: {}, Address: {})",
        id.to_string(),
        body.product,
        body.price,
        body.address
    )
}

#[launch]
fn rocket() -> _ {
    let coord = Coordinator {
        processes: vec![
            Box::new(processes::InventoryClient::default()),
            Box::new(processes::DeliveryClient::default()),
            Box::new(processes::PaymentClient::default())
        ],
        ..Default::default()
    };
    rocket::build()
        .configure(rocket::Config::figment().merge(("port", 8090)))
        .manage(coord)
        .mount("/", routes![order])
}
