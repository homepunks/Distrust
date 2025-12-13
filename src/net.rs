use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use uuid::Uuid;

use crate::data;

pub async fn handle_client(mut stream: TcpStream, addr: SocketAddr, db: Arc<data::Database>) -> io::Result<()> {
    let db = db.clone();
    let mut buf = [0u8; 1024];
    loop {
	stream.write_all(b"Choose mode: GET | POST:\t").await?;
	let n = stream.read(&mut buf).await?;
	if n == 0 {
	    println!("[INFO] Client {addr} disconnected.");
	    break;
	}

	let cmd = String::from_utf8_lossy(&buf[..n-1]).to_string();
	buf[..n].fill(0);

	match cmd.to_uppercase().trim() {
	    "GET" => {
		stream.write_all(b"Processing your GET request...\n").await?;
		process_get(&mut stream, &mut buf, &db, &addr).await?;
	    },
	    "POST" => {
		stream.write_all(b"Processing your POST request...\n").await?;
		process_post(&mut stream, &mut buf, &db, &addr).await?;
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

async fn process_get(stream: &mut TcpStream, mut buf: &mut [u8], db: &Arc<data::Database>, addr: &SocketAddr) -> io::Result<()> {
    stream.write_all(b"Enter resource's UID: ").await?;
    
    let n = stream.read(&mut buf).await?;
    if n == 0 {
	println!("[INFO] Client {addr} disconnected.");
	return Ok(());
    }

    let uid = String::from_utf8_lossy(&buf[..n-1]).trim().to_string();
    match db.get_resource(&uid).await {
	Ok(Some(resource)) => {
	    let response = format!("\nID: {}\nContent: {}\n", resource.uid, resource.content);
	    stream.write_all(response.as_bytes()).await?;
	}
	Ok(None) => {
	    let response = format!("\nNo data associated with ID: {}", uid);
	    stream.write_all(response.as_bytes()).await?;
	}
	Err(e) => {
	    eprintln!("[ERROR] Database error: {}", e);
	    stream.write_all(b"\nCould not retrieve data.\n").await?;
	}
    }
    
    Ok(())
}

async fn process_post(stream: &mut TcpStream, mut buf: &mut [u8], db: &Arc<data::Database>, addr: &SocketAddr) -> io::Result<()> {
    stream.write_all(b"Enter text to be saved: ").await?;

    let n = stream.read(&mut buf).await?;
    if n == 0 {
	println!("[INFO] Client {addr} disconnected.");
	return Ok(());
    }
    
    let content = String::from_utf8_lossy(&buf[..n-1]).trim().to_string();
    let uid = Uuid::new_v4().to_string();

    match db.post_resource(&uid, &content).await {
	Ok(()) => {
	    let response = format!("\nResource saved successfully with ID: {}", uid);
	    stream.write_all(response.as_bytes()).await?;
	}
	Err(e) => {
	    /* if collision actually happens... */
	    eprintln!("[ERROR] Database error: {}", e);

	    let uid = Uuid::new_v4().to_string();
	    match db.post_resource(&uid, &content).await {
		Ok(()) => {
		    let response = format!("\nResource saved successfully with ID: {}", uid);
		    stream.write_all(response.as_bytes()).await?;
		}
		Err(e) => {
		    eprintln!("[ERROR] Repeated database error: {}", e);
		    stream.write_all(b"Failed to save resource.\n").await?;
		}
	    }
	}
    }
    
	
    Ok(())
}
