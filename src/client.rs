use anyhow::Result;
use std::io::Write;
use std::net::TcpStream;

fn main() -> Result<()> {
    env_logger::init();
    let mut stream = TcpStream::connect("127.0.0.1:50030")?;
    let my_text = "Hello! This is test message from my sample client!";
    stream.write_all(my_text.as_bytes())?;
    Ok(())
}
