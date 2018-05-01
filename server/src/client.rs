use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_io::io::WriteHalf;
use uuid::Uuid;

#[derive(Debug)]
pub struct Client {
    id: Uuid,
    addres: SocketAddr,
    writer: WriteHalf<TcpStream>,
    listening_to: Vec<String>,
}
