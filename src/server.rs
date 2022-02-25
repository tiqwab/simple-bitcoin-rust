use anyhow::Result;
use clap::Parser;
use simple_bitcoin::core::ServerCore;
use std::net::SocketAddr;

/// Simple Bitcoin server
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    listen_addr: SocketAddr,
    #[clap(short, long)]
    core_addr: Option<SocketAddr>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();

    let mut core = ServerCore::new(args.listen_addr, args.core_addr);
    core.start().await;
    core.join_network().await;

    tokio::time::sleep(std::time::Duration::from_secs(60)).await;

    Ok(())
}
