use actix_web::{web, App, HttpServer};
use anyhow::{anyhow, Result};
use clap::Parser;
use client_core::ClientCore;
use futures::StreamExt;
use log::info;
use rand::rngs::OsRng;
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use simple_bitcoin::blockchain::utxo::UTXOManager;
use simple_bitcoin::key_manager::KeyManager;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::{Arc, Mutex};

mod api;
pub mod client_core;

/// Simple Bitcoin client
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    listen_addr: String,
    #[clap(short, long)]
    core_addr: String,
    #[clap(short, long)]
    api_addr: String,
}

async fn handle_signals(mut signals: Signals) {
    while let Some(signal) = signals.next().await {
        match signal {
            SIGTERM | SIGINT | SIGQUIT => {
                break;
            }
            _ => unreachable!(),
        }
    }
}

fn convert_to_addr(s: String) -> Result<SocketAddr> {
    s.to_socket_addrs()?
        .find(|x| x.is_ipv4())
        .ok_or(anyhow!("Illegal address: {}", s))
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let signals = Signals::new(&[SIGTERM, SIGINT, SIGQUIT])?;
    let handle = signals.handle();
    let signal_task = tokio::spawn(handle_signals(signals));

    let args = Args::parse();
    let listen_addr = convert_to_addr(args.listen_addr)?;
    let core_addr = convert_to_addr(args.core_addr)?;
    let api_addr = convert_to_addr(args.api_addr)?;

    let core = Arc::new(Mutex::new(ClientCore::new(listen_addr, core_addr)));
    core.lock().unwrap().start().await;

    let rng = OsRng;
    let key_manager = Mutex::new(KeyManager::new(rng.clone()).unwrap());
    let utxo_manager = Mutex::new(UTXOManager::new(key_manager.lock().unwrap().get_address()));

    info!("api binds at {}", api_addr);
    let app_data = web::Data::new(api::AppState::new(
        Arc::clone(&core),
        key_manager,
        utxo_manager,
    ));
    HttpServer::new(move || App::new().configure(api::config).app_data(app_data.clone()))
        .bind((api_addr.ip().to_string(), api_addr.port()))?
        .run()
        .await?;

    signal_task.await?;
    info!("Stop client");
    handle.close();

    core.lock().unwrap().shutdown().await;

    Ok(())
}
