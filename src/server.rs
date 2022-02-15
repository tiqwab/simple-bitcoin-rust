use anyhow::Result;
use std::net::SocketAddr;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};

async fn handler(mut stream: TcpStream, addr: SocketAddr) {
    println!("Connected by {:?}", addr);

    let mut buf = vec![];
    stream.read_to_end(&mut buf).await.unwrap();

    let data = String::from_utf8(buf).unwrap();
    println!("{}", data);
}

async fn run() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:50030").await?;
    println!("My ip address is {:?}", listener.local_addr()?);

    loop {
        let (stream, addr) = listener.accept().await?;
        tokio::spawn(handler(stream, addr));
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    run().await
}
