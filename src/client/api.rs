use crate::ClientCore;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use anyhow::Result;
use chrono::Utc;
use log::{debug, error, warn};
use serde::{Deserialize, Serialize};
use serde_json::json;
use simple_bitcoin::blockchain::transaction::{
    Address, NormalTransaction, TransactionInput, TransactionOutput,
};
use simple_bitcoin::blockchain::utxo::UTXOManager;
use simple_bitcoin::key_manager::KeyManager;
use simple_bitcoin::message::ApplicationPayload;
use std::future::Future;
use std::sync::{Arc, Mutex};

pub struct AppState {
    core: Arc<Mutex<ClientCore>>,
    key_manager: Mutex<KeyManager>,
    utxo_manager: Arc<Mutex<UTXOManager>>,
}

impl AppState {
    pub fn new(
        core: Arc<Mutex<ClientCore>>,
        key_manager: Mutex<KeyManager>,
        utxo_manager: Arc<Mutex<UTXOManager>>,
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

#[post("/update-balance")]
async fn request_update_balance(state: web::Data<AppState>) -> impl Responder {
    let payload = ApplicationPayload::RequestFullChain;
    let core = state.core.lock().unwrap();
    core.send_msg_to_core(payload).await;
    HttpResponse::Ok()
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

#[derive(Deserialize, Serialize, Debug)]
struct PostTransactionRequest {
    recipient: String,
    value: u64,
    fee: u64,
}

#[post("/transaction")]
async fn post_transaction(
    req: web::Json<PostTransactionRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let mut utxo_manager = state.utxo_manager.lock().unwrap();

    let tx = match utxo_manager.create_transaction_for(req.recipient.clone(), req.value, req.fee) {
        Ok(tx) => tx,
        Err(err) => {
            warn!("post_transaction failed: {:?}", err);
            return HttpResponse::BadRequest().json(json!({"error": "Failed to process request."}));
        }
    };

    let payload = ApplicationPayload::NewTransaction { transaction: tx };
    state.core.lock().unwrap().send_msg_to_core(payload).await;
    HttpResponse::Created().finish()
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(hello)
        .service(get_balance)
        .service(request_update_balance)
        .service(generate_block)
        .service(post_transaction);
}
