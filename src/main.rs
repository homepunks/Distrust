use std::io;
use std::env;
use std::sync::Arc;
use tokio::net::TcpListener;
use crate::data::Database;

mod net;
mod data;

#[tokio::main]
async fn main() -> io::Result<()> {
    let db_path = env::current_dir()?
	.join("data/resources.db");
    
    let db = Database::connect(&db_path)
	.await
	.map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    let db = Arc::new(db);
    
    let host = "0.0.0.0:6969";
    let listener = TcpListener::bind(host).await?;
    println!("[INFO] Server started and listening on {host}");

    loop {
	match listener.accept().await {
	    Ok((stream, addr)) => {
		println!("[INFO] Accepted connection from: {addr}");
		let db = db.clone();
		
		tokio::spawn(async move {
		    if let Err(e) = net::handle_client(stream, addr, db).await {
			eprintln!("[ERROR] Could not handle client {addr}: {e}");
		    }
		});
	    }
	    Err(e) => eprintln!("ERROR: Could not accept connection: {e}"),
	}
    }
}
