use std::io;
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;

pub async fn handle_client(mut stream: TcpStream) -> io::Result<()> {
    stream.write_all(b"Bonjour!\n").await?;
    
    Ok(())
}
