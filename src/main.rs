use std::{net::SocketAddr, collections::HashMap, sync::Arc};

use mac_address::MacAddressError;
use tokio::{net::{UdpSocket, TcpListener, TcpStream}, sync::RwLock, stream, io::{AsyncReadExt, AsyncWriteExt}};

mod structs;
use crate::structs::{KeyValueStore, Message, NodeInfo};

const BROADCAST_ADDR: &str = "255.255.255.255:8888";
const TCP_PORT: u16 = 9000;

#[tokio::main]
async fn main()-> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");
    let local_addr: SocketAddr = "0.0.0.0:8888".parse()?;
    let socket = UdpSocket::bind(&local_addr).await?;
    socket.set_broadcast(true)?;

    let key_value_store = Arc::new(KeyValueStore::new());

    let nodes = Arc::new(RwLock::new(HashMap::<String, NodeInfo>::new()));

    let socket = Arc::new(socket);
    let socket_for_broadcast = socket.clone();

    tokio::spawn(async move {
        match get_mac_address() {
            Ok(node_name) => {
                let tcp_addr = format!("{}:{}", "0.0.0.0", TCP_PORT).parse().unwrap();
                let msg = Message::Handshake {
                    node_name: node_name.clone(),
                    tcp_address: tcp_addr
                };
                let serialized_message = serde_json::to_string(&msg).unwrap();
                loop {
                    println!("Sending UDP broadcast...");
                    socket_for_broadcast.send_to(serialized_message.as_bytes(), BROADCAST_ADDR).await.unwrap();
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            },
            Err(error) => {
                eprintln!("Error in fetching MAC address: {:?}", error);
            }
        }
    });
    
    let nodes_clone = nodes.clone();
    tokio::spawn(async move {
        let listener = TcpListener::bind(("0.0.0.0", TCP_PORT)).await.unwrap();
        println!("TCP listener started");
        while let Ok((stream, _)) = listener.accept().await {
            println!("Accepted new TCP connection");
            tokio::spawn(handle_tcp_stream)
        }

    })
    Ok(())
} 

fn get_mac_address() -> Result<String, MacAddressError> {
    let mac = mac_address::get_mac_address()?;
    match mac {
        Some(address) => Ok(address.to_string()),
        None => Err(MacAddressError::InternalError)
    }
}

async fn handle_tcp_stream(mut stream: TcpStream, nodes: Arc<RwLock<HashMap<String, NodeInfo>>>, key_value_store: Arc<KeyValueStore>) {
    let mut buf = vec![0u8; 1024];
    let len =stream.read(&mut buf).await.unwrap();
    let received_msg: Message = serde_json::from_slice(&buf[..len]).unwrap();

    match received_msg {
        Message::Heartbeat => {
            println!("Received Heartbeat");
            let response = Message::HeartbeatResponse;
            let serialized_response = serde_json::to_string(&response).unwrap();
            stream.write_all(serialized_response.as_bytes()).await.unwrap();
        },
        Message::Setvalue { key, value } => {
            println!("Received setvalue");
            key_value_store.set(key, value).await;

            // sync to all nodes
            let nodes_guard = nodes.read().await;
            for (_, node_info) in nodes_guard.iter() {
                let mut stream = match TcpStream::connect(node_info.tcp_address).await {
                    Ok(stream) => stream,
                    Err(_) => continue
                };
            }
        }
    }
}