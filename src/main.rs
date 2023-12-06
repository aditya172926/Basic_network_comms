use std::{net::SocketAddr, collections::HashMap, sync::Arc};

use mac_address::MacAddressError;
use tokio::{net::{UdpSocket, TcpListener, TcpStream}, sync::RwLock, io::{AsyncReadExt, AsyncWriteExt}, time::Instant};

mod structs;
use crate::structs::{KeyValueStore, Message, NodeInfo};

const BROADCAST_ADDR: &str = "255.255.255.255:8888";
const TCP_PORT: u16 = 9000;

fn get_mac_address() -> Result<String, MacAddressError> {
    let mac = mac_address::get_mac_address()?;
    match mac {
        Some(address) => {
            println!("\nMac address {:?}", address.to_string());
            Ok(address.to_string())},
        None => Err(MacAddressError::InternalError)
    }
}

#[tokio::main]
async fn main()-> Result<(), Box<dyn std::error::Error>> {
    let local_addr: SocketAddr = "0.0.0.0:8888".parse()?;
    let socket: UdpSocket = UdpSocket::bind(&local_addr).await?;
    socket.set_broadcast(true)?;

    let key_value_store: Arc<KeyValueStore> = Arc::new(KeyValueStore::new());

    let nodes: Arc<RwLock<HashMap<String, NodeInfo>>> = Arc::new(RwLock::new(HashMap::<String, NodeInfo>::new()));

    let socket: Arc<UdpSocket> = Arc::new(socket);
    let socket_for_broadcast: Arc<UdpSocket> = socket.clone();
    println!("\nsocket for broadcast {:?}", socket_for_broadcast);

    tokio::spawn(async move {
        match get_mac_address() {
            Ok(node_name) => {
                let tcp_address: SocketAddr = format!("{}:{}", "0.0.0.0", TCP_PORT).parse().unwrap();
                let msg: Message = Message::Handshake {
                    node_name: node_name.clone(),
                    tcp_address
                };
                println!("\nHandshake message {:?}", msg);
                let serialized_message: String = serde_json::to_string(&msg).unwrap();
                loop {
                    socket_for_broadcast.send_to(serialized_message.as_bytes(), BROADCAST_ADDR).await.unwrap();
                    println!("\nSending UDP broadcast...");
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                }
            },
            Err(error) => {
                eprintln!("\nError in fetching MAC address: {:?}", error);
            }
        }
    });
    println!("here");
    let nodes_clone = nodes.clone();
    tokio::spawn(async move {
        let listener: TcpListener = TcpListener::bind(("0.0.0.0", TCP_PORT)).await.unwrap();
        println!("TCP listener started {:?}", listener);
        while let Ok((stream, _)) = listener.accept().await {
            println!("\nAccepted new TCP connection");
            tokio::spawn(handle_tcp_stream(stream, nodes_clone.clone(), key_value_store.clone()));
        }
    });

    // tokio::spawn(async move {
    //     loop {
    //         println!("\nTesting this spawn task\n");
    //         tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    //     }
    // });

    let mut buf = vec![0u8; 1024];
    loop {
        let (len, address) = socket.recv_from(&mut buf).await.unwrap();
        println!("\nReceived data on UDP from {}", address);
        let received_message: Message = serde_json::from_slice(&buf[..len])?;
        let local_node_name: String = get_mac_address()?;

        if let Message::Handshake{node_name, tcp_address} = received_message {
            // ignoring our own packets
            if node_name == local_node_name {
                continue;
            }
            println!("\nReceived handshake from node {:?}", node_name);
            {
                let mut nodes_guard = nodes.write().await;
                nodes_guard.insert(node_name.clone(), NodeInfo { last_seen: Instant::now(), tcp_address });
            }

            let greeting =  Message::Greeting;
            let serialized_greeting = serde_json::to_string(&greeting).unwrap();
            socket.send_to(serialized_greeting.as_bytes(), &address).await?;

            let nodes_clone = nodes.clone();
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    println!("Sending the heartbeat to {}", tcp_address);
                    let mut stream = TcpStream::connect(tcp_address).await.unwrap();
                    let heartbeat_message = Message::Heartbeat;
                    let serialize_message = serde_json::to_string(&heartbeat_message).unwrap();
                    stream.write_all(serialize_message.as_bytes()).await.unwrap();
                }
            });
        }
    }

    Ok(())
} 
async fn handle_tcp_stream(mut stream: TcpStream, nodes: Arc<RwLock<HashMap<String, NodeInfo>>>, key_value_store: Arc<KeyValueStore>) {
    let mut buf: Vec<u8> = vec![0u8; 1024];
    let len: usize =stream.read(&mut buf).await.unwrap();
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
            key_value_store.set(key.clone(), value.clone()).await;

            // sync to all nodes
            let nodes_guard = nodes.read().await;
            for (_, node_info) in nodes_guard.iter() {
                let mut stream = match TcpStream::connect(node_info.tcp_address).await {
                    Ok(stream) => stream,
                    Err(_) => continue
                };
                let sync_msg = Message::Sync { key: key.clone(), value: value.clone() };
                let serialized_message = serde_json::to_string(&sync_msg).unwrap();
                let _ = stream.write_all(serialized_message.as_bytes()).await;
            }
            let response = Message::ValueResponse { value: Some("Value set successful".to_string()) };
            let serialized_response = serde_json::to_string(&response).unwrap();
            stream.write_all(serialized_response.as_bytes()).await.unwrap();
        },
        Message::Getvalue { key } => {
            println!("Received Getvalue");
            let value = key_value_store.get(&key).await;
            let response = Message::ValueResponse { value };
            let serialize_response = serde_json::to_string(&response).unwrap();
            stream.write_all(serialize_response.as_bytes()).await.unwrap();
        },
        Message::Sync { key, value } => {
            println!("Receive sync");
            key_value_store.set(key, value).await;
        },
        _ => {}

    }
}