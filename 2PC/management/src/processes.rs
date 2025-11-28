use std::io::{Error, ErrorKind};
use async_trait::async_trait;
use reqwest::{Client};
use rocket::serde::{Deserialize, Serialize};
use crate::{OrderRequest, Process};

#[derive(Serialize, Deserialize)]
struct IDResponse {
    id: String,
}

#[derive(Serialize, Deserialize)]
struct InventoryRequest {
    quantity: usize,
}

pub(crate) struct InventoryClient {
    base_url: String,
    client: Client
}

impl Default for InventoryClient {
    fn default() -> Self {
        InventoryClient{
            base_url: String::from("http://localhost:7050/"),
            client: Client::new()
        }
    }
}

#[async_trait]
impl Process for InventoryClient {
    async fn prepare(&self, request: OrderRequest) -> Result<String, Error> {
        let res = self.client
            .post(format!(
                "{}/reserve/{}",
                self.base_url,
                request.product
            ))
            .json(&InventoryRequest { quantity: request.quantity })
            .send()
            .await;

        if res.is_err() {
            return Err(Error::new(ErrorKind::Other, res.err().unwrap().to_string()));
        }
        let r = res.ok().unwrap().json::<IDResponse>().await.unwrap();
        Ok(r.id)
    }

    async fn commit(&self, id: String) {
        self.client
            .post(format!(
                "{}/commit/{}",
                self.base_url,
                id
            ))
            .send()
            .await.expect("Unexpected commit error");
    }

    async fn rollback(&self, id: String) {
        todo!()
    }
}

pub(crate) struct DeliveryClient {
    base_url: String,
    client: Client
}

impl Default for DeliveryClient {
    fn default() -> Self {
        DeliveryClient{
            base_url: String::from("http://localhost:7070/"),
            client: Client::new()
        }
    }
}

#[derive(Serialize, Deserialize)]
struct DeliveryRequest {
    address: String
}

#[async_trait]
impl Process for DeliveryClient {
    async fn prepare(&self, request: OrderRequest) -> Result<String, Error> {
        let res = self.client
            .post(format!(
                "{}/schedule",
                self.base_url,
            ))
            .json(&DeliveryRequest { address: request.address.clone() })
            .send()
            .await;

        if res.is_err() {
            return Err(Error::new(ErrorKind::Other, res.err().unwrap().to_string()));
        }
        let r = res.ok().unwrap().json::<IDResponse>().await.unwrap();
        Ok(r.id)
    }

    async fn commit(&self, id: String) {
        self.client
            .post(format!(
                "{}/confirm/{}",
                self.base_url,
                id
            ))
            .send()
            .await.expect("Unexpected commit error");
    }

    async fn rollback(&self, id: String) {
        todo!()
    }
}


pub(crate) struct PaymentClient {
    base_url: String,
    client: Client
}

impl Default for PaymentClient {
    fn default() -> Self {
        PaymentClient{
            base_url: String::from("http://localhost:8060/"),
            client: Client::new()
        }
    }
}

#[derive(Serialize, Deserialize)]
struct PaymentRequest {
    account: usize,
    value: usize
}

#[async_trait]
impl Process for PaymentClient {
    async fn prepare(&self, request: OrderRequest) -> Result<String, Error> {
        let res = self.client
            .post(format!(
                "{}/payment",
                self.base_url,
            ))
            .json(&PaymentRequest { account: 10, value: request.price as usize })
            .send()
            .await;

        if res.is_err() {
            return Err(Error::new(ErrorKind::Other, res.err().unwrap().to_string()));
        }
        let r = res.ok().unwrap().json::<IDResponse>().await.unwrap();
        Ok(r.id)
    }

    async fn commit(&self, _: String) {
        // Do nothing, because service cannot reserve
    }

    async fn rollback(&self, id: String) {
        todo!()
    }
}
