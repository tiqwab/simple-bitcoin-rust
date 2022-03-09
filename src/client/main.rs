use actix_web::{web, App, HttpServer};
use anyhow::{anyhow, Result};
use clap::Parser;
use client_core::ClientCore;
use futures::StreamExt;
use log::info;
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;
use std::net::{SocketAddr, ToSocketAddrs};

pub mod client_core;

/// Simple Bitcoin client
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    listen_addr: String,
    #[clap(short, long)]
    core_addr: String,
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

async fn hello() -> &'static str {
    "hello"
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

    let mut core = ClientCore::new(listen_addr, core_addr);
    core.start().await;

    HttpServer::new(|| App::new().route("/hello", web::get().to(hello)))
        .bind(("127.0.0.1", 12345))?
        .run()
        .await?;

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    core.send_message_to_my_core_node().await;

    signal_task.await?;
    info!("Stop client");
    handle.close();

    core.shutdown().await;

    Ok(())
}
