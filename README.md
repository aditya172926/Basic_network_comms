# Basic implementation of TCP/IP Protocol using Rust

The project implements a live connection between the peers using the software on the same network to share data among the nodes impersonating as a distributed database in a torrent like system

Each peer in the network is identified by its machine address derived by the `get_mac_address` function. 
A socket connection is made by binding a UDP socket to all available interfaces on 0.0.0.0 and port 9000. 

The p2p system will act like a distributed database storing key-value pairs.

For concurrent access in the HashMap and to prevent a Data Race condition between multiple threads we wrap the HashMap with RwLock.

The information about the peer like its TCP address and last login time is also stored. 

## Starting up

A UDP signal is used to accounce the presence of a live node on the broadcasting address. This allows the other live node on the same network to discover and connect to the new peer.

A handshake signal is broadcasted by other nodes which have discovered the new peers MAC address and store this information. 

After the connection the rest of the communication is done using TCP protocol between the peers. `tokio::spawn` is used to handle new tasks for each connection.

## Basic Message Type
- Heartbeat message - This is a signal given between the connected peers which lets them know that the peers are live and functioning, or offline. It is accompanied by a Response from the receiving peer
- SetValue - Sends a key-value to store in the memory and also lets other peers know to sync their storages with the latest change
- GetValue - requires a key parameter and returns the corresponding stored value
- Greeting message - upon receiving a handshake UDP signal, a greeting message is sent
