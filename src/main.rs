use std::io;
use tokio::net::TcpListener;

mod net;

#[tokio::main]
async fn main() -> io::Result<()> {
    let host = "0.0.0.0:6969";
    let listener = TcpListener::bind(host).await?;
    println!("[INFO] Server started and listening on {host}");

    loop {
	match listener.accept().await {
	    Ok((stream, addr)) => {
		println!("[INFO] Accepted connection from: {addr}");
		
		tokio::spawn(async move {
		    if let Err(e) = net::handle_client(stream, addr).await {
			eprintln!("[ERROR] Could not handle client {addr}: {e}");
		    }
		});
	    }
	    Err(e) => eprintln!("ERROR: Could not accept connection: {e}"),
	}
    }
}
