use anyhow::Result;
use clap::Parser;
use futures::StreamExt;
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use simple_bitcoin::blockchain::manager::BlockchainManager;
use simple_bitcoin::blockchain::transaction_pool::TransactionPool;
use simple_bitcoin::server_core::ServerCore;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Simple Bitcoin server
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    listen_addr: SocketAddr,
    #[clap(short, long)]
    core_addr: Option<SocketAddr>,
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

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let signals = Signals::new(&[SIGTERM, SIGINT, SIGQUIT])?;
    let handle = signals.handle();
    let signal_task = tokio::spawn(handle_signals(signals));

    let bm = Arc::new(Mutex::new(BlockchainManager::new()));
    let tp = Arc::new(Mutex::new(TransactionPool::new()));

    tokio::spawn(TransactionPool::generate_block_periodically(
        Arc::clone(&tp),
        Arc::clone(&bm),
        Duration::from_secs(10),
    ));

    let args = Args::parse();
    let mut core = ServerCore::new(args.listen_addr, args.core_addr, tp, bm);
    core.start().await;
    core.join_network().await;

    signal_task.await?;
    handle.close();

    core.shutdown().await;

    Ok(())
}
