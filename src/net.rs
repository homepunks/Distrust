use std::io;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};

pub async fn handle_client(mut stream: TcpStream, addr: SocketAddr) -> io::Result<()> {
    let mut buf = [0u8; 256];
    loop {
	stream.write_all(b"Choose mode: GET | POST:\t").await?;
	let n = stream.read(&mut buf).await?;
	if n == 0 {
	    println!("[INFO] Client {addr} disconnected.");
	    break;
	}

	let cmd = String::from_utf8_lossy(&buf[..n-1]).to_string();

	match cmd.to_uppercase().trim() {
	    "GET" => {
		stream.write_all(b"Processing your GET request...\n").await?;
		process_get(&mut stream).await?;
	    },
	    "POST" => {
		stream.write_all(b"Processing your POST request...\n").await?;
		process_post(&mut stream).await?;
	    },
	    "EXIT" => {
		stream.write_all(b"Exiting...\n").await?;
		println!("[INFO] Client {addr} disconnected.");
		break;
	    },
	    _ => stream.write_all(b"Unavailable command!\n").await?,
	}

	println!("[INFO] Client {addr} requested {}.", cmd.trim());
    }
    

    Ok(())
}

async fn process_get(stream: &mut TcpStream) -> io::Result<()>{
    stream.write_all(b"GET reached.\n").await?;
    Ok(())
}

async fn process_post(stream: &mut TcpStream) -> io::Result<()> {
    stream.write_all(b"POST reached.\n").await?;
    Ok(())
}
