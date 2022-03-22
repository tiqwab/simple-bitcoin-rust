use anyhow::{anyhow, Result};
use clap::Parser;
use futures::StreamExt;
use rand::rngs::OsRng;
use server_core::ServerCore;
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use simple_bitcoin::blockchain::manager::BlockchainManager;
use simple_bitcoin::blockchain::transaction_pool::TransactionPool;
use simple_bitcoin::key_manager::KeyManager;
use std::net::{SocketAddr, ToSocketAddrs};
use std::sync::{Arc, Mutex};

pub mod server_core;

/// Simple Bitcoin server
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    listen_addr: String,
    #[clap(short, long)]
    core_addr: Option<String>,
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
        .ok_or_else(|| anyhow!("Illegal address: {}", s))
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let signals = Signals::new(&[SIGTERM, SIGINT, SIGQUIT])?;
    let handle = signals.handle();
    let signal_task = tokio::spawn(handle_signals(signals));

    let rng = OsRng;

    let bm = Arc::new(Mutex::new(BlockchainManager::new(3)));
    let tp = Arc::new(Mutex::new(TransactionPool::new()));
    let km = Arc::new(Mutex::new(KeyManager::new(rng).unwrap()));

    let args = Args::parse();
    let listen_addr = convert_to_addr(args.listen_addr)?;
    let core_addr = args.core_addr.map(|x| convert_to_addr(x).unwrap());

    let mut core = ServerCore::new(listen_addr, core_addr, tp, bm, km);
    core.start().await;
    core.join_network().await;

    signal_task.await?;
    handle.close();

    core.shutdown().await;

    Ok(())
}
