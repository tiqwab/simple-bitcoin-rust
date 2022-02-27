use anyhow::Result;
use clap::Parser;
use futures::StreamExt;
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use simple_bitcoin::client_core::ClientCore;
use std::net::SocketAddr;

/// Simple Bitcoin client
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    listen_addr: SocketAddr,
    #[clap(short, long)]
    core_addr: SocketAddr,
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

    let args = Args::parse();
    let mut core = ClientCore::new(args.listen_addr, args.core_addr);
    core.start().await;

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    core.send_message_to_my_core_node().await;

    signal_task.await?;
    handle.close();

    core.shutdown().await;

    Ok(())
}
