use crate::ClientCore;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use simple_bitcoin::blockchain::utxo::UTXOManager;
use simple_bitcoin::key_manager::KeyManager;
use simple_bitcoin::message::ApplicationPayload;
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

// test api to generate UTXO for myself
#[post("/generate-block")]
async fn generate_block(state: web::Data<AppState>) -> impl Responder {
    let addr = state.key_manager.lock().unwrap().get_address();
    let payload = ApplicationPayload::Enhanced {
        data: addr.as_bytes().to_owned(),
    };
    state.core.lock().unwrap().send_msg_to_core(payload).await;
    HttpResponse::Ok()
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(hello)
        .service(get_balance)
        .service(generate_block);
}
