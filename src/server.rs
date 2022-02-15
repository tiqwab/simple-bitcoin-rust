use anyhow::Result;
use std::io::Read;
use std::net::TcpListener;

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:50030")?;
    println!("My ip address is {:?}", listener.local_addr()?);

    for conn in listener.incoming() {
        let mut conn = conn.unwrap();
        println!("Connected by {:?}", conn.peer_addr()?);
        let mut buf = vec![];
        conn.read_to_end(&mut buf)?;
        let data = String::from_utf8(buf)?;
        println!("{}", data);
    }

    Ok(())
}
