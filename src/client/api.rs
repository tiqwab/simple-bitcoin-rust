use crate::ClientCore;
use actix_web::{get, web, App, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use simple_bitcoin::blockchain::utxo::UTXOManager;
use simple_bitcoin::key_manager::KeyManager;
use std::sync::{Arc, Mutex};

pub struct AppState {
    core: Arc<Mutex<ClientCore>>,
    key_manager: Mutex<KeyManager>,
    utxo_manager: Mutex<UTXOManager>,
}

impl AppState {
    pub fn new(
        core: Arc<Mutex<ClientCore>>,
        key_manager: Mutex<KeyManager>,
        utxo_manager: Mutex<UTXOManager>,
    ) -> AppState {
        AppState {
            core,
            key_manager,
            utxo_manager,
        }
    }
}

#[get("/hello")]
async fn hello() -> impl Responder {
    "hello"
}

#[derive(Deserialize, Serialize)]
struct GetBalanceResponse {
    balance: u64,
}

impl GetBalanceResponse {
    fn new(balance: u64) -> GetBalanceResponse {
        GetBalanceResponse { balance }
    }
}

#[get("/balance")]
async fn get_balance(state: web::Data<AppState>) -> impl Responder {
    let balance = state.utxo_manager.lock().unwrap().get_balance();
    web::Json(GetBalanceResponse::new(balance))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(hello).service(get_balance);
}
